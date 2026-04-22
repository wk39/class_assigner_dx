#![allow(non_snake_case)]

use dioxus::prelude::*;
use rand::{rngs::StdRng, seq::SliceRandom, Rng as _, SeedableRng as _};
use rand_distr::{Distribution as _, Normal};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

// --------------------
// 1. 라우트 정의
// --------------------
#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    // 모든 하위 페이지에 SidebarLayout 적용
    #[layout(SidebarLayout)]
    //
    #[route("/")]
    MainPage {},

    #[route("/student-list")]
    StudentList {}, // 작업 페이지 1
    //
    #[route("/assign-class")]
    AssignClass {}, // 작업 페이지 2
    //
    #[route("/assign-result")]
    ResultDetail {},
    //
    #[route("/info")]
    InfoPage {},

    // #[route("/egui-viewer")]
    // EguiPage {},
    #[end_layout]
    //
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");
    launch(App);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Gender {
    Male,
    Female,
}
impl Gender {
    fn to_label(&self) -> &'static str {
        match self {
            Gender::Male => "남",
            Gender::Female => "여",
        }
    }
    fn to_value(&self) -> &'static str {
        match self {
            Gender::Male => "male",
            Gender::Female => "female",
        }
    }
    // HTML String -> Enum 변환
    fn from_str(s: &str) -> Self {
        match s {
            "female" => Gender::Female,
            _ => Gender::Male, // 기본값
        }
    }
}

pub(crate) type StudentId = u32;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Student {
    pub(crate) id: StudentId,
    pub(crate) name: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) gender: Gender,
    pub(crate) score: f32,
    pub(crate) valid: bool, // for convenience; if true, this is not a real student.
}
impl Student {
    pub(crate) fn new(
        id: impl Into<StudentId>,
        name: Option<String>,
        gender: Gender,
        score: f32,
    ) -> Self {
        Self {
            id: id.into(),
            name,
            gender,
            note: None,
            score,
            valid: false,
        }
    }

    pub(crate) fn new_dummy(id: impl Into<StudentId>) -> Student {
        Self {
            id: id.into(),
            name: None,
            gender: Gender::Male,
            note: None,
            score: 0.0,
            valid: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ClassInfo {
    pub(crate) id: u8,
    pub(crate) students: Vec<Student>,
}
impl ClassInfo {
    fn avg_score(&self) -> f32 {
        if self.students.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.students.iter().map(|s| s.score).sum();
        sum / self.students.len() as f32
    }
    fn count_gender(&self, g: Gender) -> usize {
        self.students.iter().filter(|s| s.gender == g).count()
    }
}

fn assign_classes(
    students: &[Student],
    k: u8,
    opt_score: bool,
    opt_gender: bool,
) -> Vec<ClassInfo> {
    let k = (k as usize).max(1);
    let mut classes: Vec<ClassInfo> = (1..=k as u8)
        .map(|id| ClassInfo {
            id,
            students: Vec::new(),
        })
        .collect();

    let groups: Vec<Vec<Student>> = if opt_gender {
        vec![
            students
                .iter()
                .filter(|s| s.gender == Gender::Male)
                .cloned()
                .collect(),
            students
                .iter()
                .filter(|s| s.gender == Gender::Female)
                .cloned()
                .collect(),
        ]
    } else {
        vec![students.to_vec()]
    };

    let mut rng = StdRng::from_os_rng();
    for mut group in groups {
        if opt_score {
            group.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            group.shuffle(&mut rng);
        }
        for (idx, student) in group.into_iter().enumerate() {
            let round = idx / k;
            let pos = idx % k;
            let class_idx = if round % 2 == 0 { pos } else { k - 1 - pos };
            classes[class_idx].students.push(student);
        }
    }

    classes
}

// --------------------
// CSV helpers
// --------------------

fn escape_csv(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn students_to_csv(students: &[Student]) -> String {
    let mut out = String::from("\u{FEFF}name,gender,score,note\n");
    for s in students {
        let name = s.name.clone().unwrap_or_default();
        let gender = s.gender.to_label();
        let note = s.note.clone().unwrap_or_default();
        out.push_str(&format!(
            "{},{},{:.2},{}\n",
            escape_csv(&name),
            gender,
            s.score,
            escape_csv(&note),
        ));
    }
    out
}

fn assignments_to_csv(classes: &[ClassInfo]) -> String {
    let mut out = String::from("\u{FEFF}class,id,name,gender,score,note\n");
    for c in classes {
        for s in &c.students {
            let name = s.name.clone().unwrap_or_default();
            let gender = s.gender.to_label();
            let note = s.note.clone().unwrap_or_default();
            out.push_str(&format!(
                "{},{},{},{},{:.2},{}\n",
                c.id,
                s.id,
                escape_csv(&name),
                gender,
                s.score,
                escape_csv(&note),
            ));
        }
    }
    out
}

fn encode_uri_component(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn csv_data_url(csv: &str) -> String {
    format!("data:text/csv;charset=utf-8,{}", encode_uri_component(csv))
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => out.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

fn looks_like_header(cols: &[String]) -> bool {
    cols.iter().any(|c| {
        matches!(
            c.trim().to_lowercase().as_str(),
            "name"
                | "이름"
                | "성명"
                | "gender"
                | "성별"
                | "score"
                | "점수"
                | "note"
                | "비고"
                | "class"
                | "반"
                | "id"
        )
    })
}

fn parse_students_csv(content: &str, start_id: u32) -> Vec<Student> {
    let content = content.trim_start_matches('\u{FEFF}');
    let mut students = Vec::new();
    let mut next_id = start_id;

    let (name_idx, gender_idx, score_idx, note_idx, data_lines): (
        usize,
        usize,
        usize,
        usize,
        Vec<&str>,
    ) = {
        let mut lines = content.lines();
        let first = lines.clone().next();
        if let Some(first_line) = first {
            let cols = parse_csv_line(first_line);
            if looks_like_header(&cols) {
                let mut ni = 0usize;
                let mut gi = 1usize;
                let mut sci = 2usize;
                let mut noi = 3usize;
                for (i, c) in cols.iter().enumerate() {
                    match c.trim().to_lowercase().as_str() {
                        "name" | "이름" | "성명" => ni = i,
                        "gender" | "성별" => gi = i,
                        "score" | "점수" => sci = i,
                        "note" | "비고" => noi = i,
                        _ => {}
                    }
                }
                lines.next();
                (ni, gi, sci, noi, lines.collect())
            } else {
                (0, 1, 2, 3, content.lines().collect())
            }
        } else {
            (0, 1, 2, 3, Vec::new())
        }
    };

    for line in data_lines {
        let cols = parse_csv_line(line);
        if cols.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let name = cols.get(name_idx).and_then(|s| {
            let t = s.trim();
            (!t.is_empty()).then(|| t.to_string())
        });
        let gender = cols
            .get(gender_idx)
            .map(|s| match s.trim().to_lowercase().as_str() {
                "여" | "female" | "f" => Gender::Female,
                _ => Gender::Male,
            })
            .unwrap_or(Gender::Male);
        let score = cols
            .get(score_idx)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(0.0);
        let note = cols.get(note_idx).and_then(|s| {
            let t = s.trim();
            (!t.is_empty()).then(|| t.to_string())
        });

        students.push(Student {
            id: next_id,
            name,
            note,
            gender,
            score,
            valid: false,
        });
        next_id += 1;
    }

    students
}

#[derive(Clone)]
struct AppState {
    number_of_class: u8,
    //
    students: Vec<Student>,
    next_student_id: u32,
    //
    opt_score: bool,
    opt_gender: bool,
    //
    assignments: Option<Vec<ClassInfo>>,
    //
    consented: bool,
}

fn App() -> Element {
    //

    // let mut rng = StdRng::seed_from_u64(0);
    let mut rng = StdRng::from_os_rng();
    let normal = Normal::new(60.0_f32, 15.0).expect("get random(normal) failed");

    let n_students: u32 = 101;
    let students: Vec<_> = (0..n_students)
        .map(|iid| {
            let gender = if rng.random_bool(0.5) {
                Gender::Male
            } else {
                Gender::Female
            };

            let score = normal.sample(&mut rng).clamp(0.0, 100.0);

            Student::new(iid, None, gender, score)
            //
        })
        .collect();

    use_context_provider(move || {
        Signal::new(AppState {
            number_of_class: 10,
            //
            students,
            next_student_id: n_students,

            //
            opt_score: true,
            opt_gender: true,
            //
            assignments: None,
            //
            consented: false,
        })
    });

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        // style { "{CSS_STYLE}" }

        // 전역 CSS 스타일 주입
        Router::<Route> {}
    }
}

// --------------------
// 2. 레이아웃 컴포넌트 (사이드바)
// --------------------
fn SidebarLayout() -> Element {
    let state = use_context::<Signal<AppState>>();
    if !state.read().consented {
        return rsx! { ConsentScreen {} };
    }
    rsx! {
        div { class: "app-container",
            // 왼쪽 사이드바
            nav { class: "sidebar",
                h2 { "Class Assigner" }
                ul {
                    li { Link { to: Route::MainPage {}, class: "nav-item", "🏠 메인 페이지" } }
                    li { Link { to: Route::StudentList {}, class: "nav-item", "📝 학생 목록" } }
                    li { Link { to: Route::AssignClass {}, class: "nav-item", "😃 반 배정" } }
                    li { Link { to: Route::ResultDetail {}, class: "nav-item", "📋 배정 결과" } }
                    li { Link { to: Route::InfoPage {}, class: "nav-item", "💁 프로그램 정보" } }
                    // li { Link { to: Route::EguiPage {}, class: "nav-item", "EGUI TEST" } }
                }
            }

            // 오른쪽 메인 콘텐츠 (페이지가 바뀌는 부분)
            main { class: "content-area",
                Outlet::<Route> {}
            }
        }
    }
}

// --------------------
// 3. 페이지 컴포넌트들
// --------------------

#[component]
fn ConsentScreen() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut checked = use_signal(|| false);

    let item_style = "margin-bottom: 14px;";

    rsx! {
        div {
            style: "min-height: 100vh; display: flex; align-items: flex-start; justify-content: center; background: #f3f4f6; padding: 40px 20px; box-sizing: border-box; overflow-y: auto;",
            div {
                style: "max-width: 720px; width: 100%; background: white; padding: 32px 36px; border-radius: 12px; box-shadow: 0 4px 12px rgba(0,0,0,0.08);",
                h1 { style: "margin-top: 0;", "이용 전 확인 및 동의" }
                p { style: "color: #4b5563;",
                    "본 프로그램은 학생 정보를 다루는 도구입니다. 시작하기 전에 아래 내용을 반드시 읽고 동의해 주세요."
                }

                ol { style: "margin: 20px 0; padding-left: 24px; line-height: 1.7; color: #1f2937;",
                    li { style: "{item_style}",
                        b { "데이터 처리 범위 · " }
                        "모든 입력과 연산은 접속한 장치의 브라우저 내에서만 수행되며, 서버나 외부로 전송되지 않습니다. 새로고침이나 탭 종료 시 입력한 데이터는 소실됩니다."
                    }
                    li { style: "{item_style}",
                        b { "개인정보 취급 책임 · " }
                        "입력하는 학생의 성명·점수·비고 등은 민감한 개인정보에 해당할 수 있습니다. "
                        b { "개인정보 보호법 등 관련 법령을 준수하여 본인의 책임 하에 취급" }
                        "해야 하며, 제3자가 접근할 수 없는 환경에서 사용하세요. 화면 캡처·인쇄물·내보낸 CSV 파일의 보관 및 폐기 또한 사용자의 책임입니다."
                    }
                    li { style: "{item_style}",
                        b { "결과에 대한 책임 · " }
                        "배정 알고리즘이 산출한 결과는 "
                        b { "참고용 보조 자료" }
                        "이며, 교육적·법적·행정적 효력을 보장하지 않습니다. 결과의 활용 및 그로 인해 발생하는 모든 사항에 대한 책임은 전적으로 사용자에게 있으며, 본 프로그램의 저작자는 어떠한 책임도 지지 않습니다."
                    }
                    li { style: "{item_style}",
                        b { "보증 부인 · " }
                        "본 소프트웨어는 현 상태(\"AS IS\")로 제공됩니다. 특정 목적 적합성·무결성·연속적 가용성 등을 보증하지 않습니다. (MIT License)"
                    }
                }

                label {
                    style: "display: flex; align-items: center; gap: 10px; padding: 12px 14px; background: #f9fafb; border: 1px solid #e5e7eb; border-radius: 8px; cursor: pointer;",
                    input {
                        r#type: "checkbox",
                        style: "width: 18px; height: 18px; flex-shrink: 0;",
                        checked: "{checked}",
                        oninput: move |evt| checked.set(evt.value() == "true"),
                    }
                    span {
                        "위 내용을 모두 확인했으며, "
                        b { "본인의 책임 하에" }
                        " 프로그램을 사용하는 것에 동의합니다."
                    }
                }

                button {
                    disabled: "{!checked()}",
                    onclick: move |_| {
                        state.write().consented = true;
                    },
                    style: if checked() {
                        "margin-top: 20px; width: 100%; padding: 14px; background: #2563eb; color: white; border: none; border-radius: 8px; font-size: 16px; font-weight: bold; cursor: pointer;"
                    } else {
                        "margin-top: 20px; width: 100%; padding: 14px; background: #9ca3af; color: white; border: none; border-radius: 8px; font-size: 16px; font-weight: bold; cursor: not-allowed;"
                    },
                    "동의하고 시작하기"
                }

                p { style: "margin-top: 16px; font-size: 13px; color: #6b7280; text-align: center;",
                    "동의하지 않을 경우 프로그램을 이용할 수 없습니다. 동의 상태는 브라우저 세션 동안만 유지됩니다."
                }
            }
        }
    }
}

#[component]
fn MainPage() -> Element {
    let state = use_context::<Signal<AppState>>();
    let student_count = state.read().students.len();
    let has_result = state.read().assignments.is_some();

    let step_card = "flex: 1; min-width: 220px; padding: 16px; background: #fff; border: 1px solid #e5e7eb; border-radius: 8px; text-decoration: none; color: inherit; display: block;";

    rsx! {
        div { style: "max-width: 800px; margin: 0 auto;",
            h1 { "학급 배정 도우미" }
            p { style: "color: #4b5563; font-size: 15px;",
                "학생 명단을 입력하면 평균 점수와 성비의 균형을 맞춰 여러 학급으로 자동 배정해 주는 도구입니다."
            }

            div {
                style: "margin-top: 20px; padding: 12px 16px; background: #fef3c7; border-left: 4px solid #f59e0b; border-radius: 4px; color: #78350f;",
                b { "⚠ 데이터 주의 " }
                "모든 작업은 접속한 장치의 브라우저 안에서만 수행됩니다. 새로고침하거나 탭을 닫으면 입력한 내용이 사라지므로, 중요한 명단은 "
                b { "CSV 내보내기" }
                " 로 저장해 두세요."
            }

            h2 { style: "margin-top: 32px;", "작업 순서" }
            div {
                style: "display: flex; gap: 12px; flex-wrap: wrap; margin-top: 12px;",
                Link {
                    to: Route::StudentList {},
                    style: "{step_card}",
                    div { style: "color: #2563eb; font-weight: 600; font-size: 13px;", "STEP 1" }
                    h3 { style: "margin: 4px 0;", "📝 학생 목록" }
                    p { style: "color: #6b7280; font-size: 14px; margin: 0;",
                        "학생 정보를 직접 입력하거나 CSV 로 가져옵니다."
                    }
                }
                Link {
                    to: Route::AssignClass {},
                    style: "{step_card}",
                    div { style: "color: #2563eb; font-weight: 600; font-size: 13px;", "STEP 2" }
                    h3 { style: "margin: 4px 0;", "😃 반 배정" }
                    p { style: "color: #6b7280; font-size: 14px; margin: 0;",
                        "학급 수와 최적화 기준을 설정하고 배정을 실행합니다."
                    }
                }
                Link {
                    to: Route::ResultDetail {},
                    style: "{step_card}",
                    div { style: "color: #2563eb; font-weight: 600; font-size: 13px;", "STEP 3" }
                    h3 { style: "margin: 4px 0;", "📋 배정 결과" }
                    p { style: "color: #6b7280; font-size: 14px; margin: 0;",
                        "결과를 확인·인쇄하거나 CSV 로 내보냅니다."
                    }
                }
            }

            h2 { style: "margin-top: 32px;", "현재 상태" }
            table { style: "border-collapse: collapse; margin-top: 8px;",
                tbody {
                    tr {
                        td { style: "padding: 4px 16px 4px 0; color: #6b7280; font-weight: 600;", "학생 수" }
                        td { style: "padding: 4px 0;", "{student_count}명" }
                    }
                    tr {
                        td { style: "padding: 4px 16px 4px 0; color: #6b7280; font-weight: 600;", "배정 결과" }
                        td { style: "padding: 4px 0;",
                            if has_result { "생성됨" } else { "아직 없음" }
                        }
                    }
                }
            }
        }
    }
}

fn StudentList() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut status_msg = use_signal(String::new);

    // 학생 추가
    let add_student = move |_| {
        let mut ss = state.write();
        let id = ss.next_student_id;
        ss.students.push(Student::new(id, None, Gender::Male, 0.0));
        ss.next_student_id += 1;
    };

    // CSV 가져오기 (교체)
    let import_csv = move |evt: FormEvent| {
        let files = evt.files();
        let Some(file) = files.into_iter().next() else {
            return;
        };
        spawn(async move {
            match file.read_string().await {
                Ok(contents) => {
                    let new_students = parse_students_csv(&contents, 0);
                    let count = new_students.len();
                    let mut ss = state.write();
                    ss.students = new_students;
                    ss.next_student_id = count as u32;
                    ss.assignments = None;
                    status_msg.set(format!("{}명을 가져왔습니다.", count));
                }
                Err(e) => {
                    status_msg.set(format!("파일을 읽을 수 없습니다: {e}"));
                }
            }
            // 동일 파일 재선택 시에도 onchange 발생하도록 value 초기화
            let _ = document::eval(
                r#"
                const el = document.getElementById('csv-import-input');
                if (el) el.value = '';
                "#,
            );
        });
    };

    // 모두 삭제
    let clear_all = move |_| {
        let mut ss = state.write();
        ss.students.clear();
        ss.next_student_id = 0;
        ss.assignments = None;
        status_msg.set("학생 목록을 비웠습니다.".to_string());
    };

    // 학생 삭제
    let mut remove_student = move |target_id: u32| {
        let mut ss = state.write();
        ss.students.retain(|s| s.id != target_id);
    };

    // 이름 수정
    let mut update_name = move |id: u32, value: String| {
        let new_name = if value.trim().is_empty() {
            None
        } else {
            Some(value)
        };
        let mut ss = state.write();
        if let Some(student) = ss.students.iter_mut().find(|s| s.id == id) {
            student.name = new_name;
        }
    };

    // 성적 수정
    let mut update_score = move |id: u32, value: String| {
        if let Ok(new_score) = value.parse::<f32>() {
            let mut ss = state.write();
            if let Some(student) = ss.students.iter_mut().find(|s| s.id == id) {
                student.score = new_score;
            }
        }
    };

    let mut update_gender = move |id: StudentId, val: String| {
        let gender = Gender::from_str(&val);
        if let Some(s) = state.write().students.iter_mut().find(|s| s.id == id) {
            s.gender = gender;
        }
    };

    let mut update_note = move |id: StudentId, val: String| {
        let new_val = if val.trim().is_empty() {
            None
        } else {
            Some(val)
        };
        if let Some(s) = state.write().students.iter_mut().find(|s| s.id == id) {
            s.note = new_val;
        }
    };
    let export_url = csv_data_url(&students_to_csv(&state.read().students));
    let student_count = state.read().students.len();

    rsx! {
        div {
            style: "max-width: 900px;",

            // 상단 버튼
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; gap: 8px; flex-wrap: wrap;",
                div { style: "color: #4b5563; font-size: 14px;", "총 {student_count}명" }
                div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                    label {
                        style: "padding: 8px 15px; background: #198754; color: white; border-radius: 4px; cursor: pointer; font-size: 14px;",
                        "📥 CSV 가져오기"
                        input {
                            id: "csv-import-input",
                            r#type: "file",
                            accept: ".csv,text/csv",
                            style: "display: none;",
                            onchange: import_csv,
                        }
                    }
                    a {
                        href: "{export_url}",
                        download: "students.csv",
                        style: "padding: 8px 15px; background: #6c757d; color: white; border-radius: 4px; text-decoration: none; font-size: 14px;",
                        "📤 CSV 내보내기"
                    }
                    button {
                        onclick: clear_all,
                        style: "padding: 8px 15px; background: #dc3545; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                        "🗑 모두 삭제"
                    }
                    button {
                        onclick: add_student,
                        style: "padding: 8px 15px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                        "+ 학생 추가"
                    }
                }
            }
            if !status_msg().is_empty() {
                div {
                    style: "margin-bottom: 10px; padding: 8px 12px; background: #e7f3ff; color: #084298; border-radius: 4px; font-size: 14px;",
                    "{status_msg}"
                }
            }

            // 테이블 시작
            table {
                style: "width: 100%; border-collapse: collapse; box-shadow: 0 0 10px rgba(0,0,0,0.1);",

                thead {
                    tr {
                        style: "background-color: #f8f9fa; text-align: left;",
                        th { style: "padding: 12px; border-bottom: 2px solid #dee2e6; width: 50px;", "ID" }
                        th { style: "padding: 12px; border-bottom: 2px solid #dee2e6;", "이름" }
                        th { style: "padding: 12px; border-bottom: 2px solid #dee2e6; width: 100px;", "성별" }
                        th { style: "padding: 12px; border-bottom: 2px solid #dee2e6; width: 80px;", "점수" }
                        th { style: "padding: 12px; border-bottom: 2px solid #dee2e6;", "비고 (Note)" }
                        th { style: "padding: 12px; border-bottom: 2px solid #dee2e6; width: 60px;", "삭제" }
                    }
                }

                tbody {
                    // .cloned()로 소유권 문제 해결 후 순회
                    for student in state.read().students.iter().cloned() {
                        tr {
                            key: "{student.id}",
                            style: "border-bottom: 1px solid #eee;",

                            // 1. ID (읽기 전용)
                            td { style: "padding: 3px 6px; text-align: center; color: #666;", "{student.id}" }

                            // 2. 이름 (Option<String>)
                            td { style: "padding: 3px 6px;",
                                input {
                                    r#type: "text",
                                    style: "width: 100%; padding: 3px 6px; border: 1px solid #ccc; border-radius: 4px;",
                                    placeholder: "이름 입력",
                                    // Some이면 값, None이면 빈 문자열
                                    value: "{student.name.clone().unwrap_or_default()}",
                                    oninput: move |evt| update_name(student.id, evt.value())
                                }
                            }

                            // 3. 성별 (Enum -> Select Box)
                            td { style: "padding: 3px 6px;",
                                select {
                                    style: "width: 100%; padding: 3px 6px; border: 1px solid #ccc; border-radius: 4px;",
                                    value: "{student.gender.to_value()}", // 현재 값 선택
                                    oninput: move |evt| update_gender(student.id, evt.value()),

                                    option { value: "male", "남" }
                                    option { value: "female", "여" }
                                }
                            }

                            // 4. 점수 (f32)
                            td { style: "padding: 3px 6px;",
                                input {
                                    r#type: "number",
                                    // step="0.1"을 주어야 소수점 입력 가능
                                    step: "0.1",
                                    style: "width: 100%; padding: 3px 6px; border: 1px solid #ccc; border-radius: 4px; text-align: right;",
                                    value: "{student.score}",
                                    oninput: move |evt| update_score(student.id, evt.value())
                                }
                            }

                            // 5. 비고 (Option<String>)
                            td { style: "padding: 3px 6px;",
                                input {
                                    r#type: "text",
                                    style: "width: 100%; padding: 3px 6px; border: 1px solid #ccc; border-radius: 4px;",
                                    placeholder: "특이사항 없음",
                                    value: "{student.note.clone().unwrap_or_default()}",
                                    oninput: move |evt| update_note(student.id, evt.value())
                                }
                            }

                            // 6. 삭제 버튼
                            td { style: "padding: 3px 6px; text-align: center;",
                                button {
                                    onclick: move |_| remove_student(student.id),
                                    style: "background: #dc3545; color: white; border: none; border-radius: 4px; padding: 3px 8px; cursor: pointer;",
                                    "X"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AssignClass() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut status_msg = use_signal(String::new);

    let run_assign = move |_| {
        let (students, k, opt_s, opt_g) = {
            let s = state.read();
            (
                s.students.clone(),
                s.number_of_class,
                s.opt_score,
                s.opt_gender,
            )
        };
        if students.is_empty() {
            status_msg.set("학생 목록이 비어 있습니다.".to_string());
            state.write().assignments = None;
            return;
        }
        let result = assign_classes(&students, k, opt_s, opt_g);
        status_msg.set(format!(
            "{}명을 {}개 반으로 배정했습니다.",
            students.len(),
            k
        ));
        state.write().assignments = Some(result);
    };

    rsx! {
        div { class: "task-container no-print",
            h1 { "반 배정" }

            h2 { "학급 수를 설정합니다." }
            input {
                style: "margin-top: 10px; margin-left: 20px;",
                r#type: "range",
                min: "3",
                max: "30",
                step: "1",
                value: "{state().number_of_class}",
                oninput: move |evt| {
                    if let Ok(val) = evt.value().parse::<u8>() {
                        state.write().number_of_class = val;
                    }
                }
            }
            span {
                style: "margin-left: 10px; font-weight: bold;",
                "{state().number_of_class} 반"
            }

            h2 { "최적화 기준을 선택합니다." }
            div {
                style: "margin-top: 10px; margin-left: 20px;",
                label {
                    style: "cursor: pointer; display: flex; align-items: center;",
                    input {
                        r#type: "checkbox",
                        style: "width: 20px; height: 20px; margin-right: 8px;",
                        checked: "{state().opt_score}",
                        oninput: move |evt| {
                            state.write().opt_score = evt.value() == "true";
                        }
                    }
                    "평균점수 균형"
                }
                label {
                    style: "cursor: pointer; display: flex; align-items: center;",
                    input {
                        r#type: "checkbox",
                        style: "width: 20px; height: 20px; margin-right: 8px;",
                        checked: "{state().opt_gender}",
                        oninput: move |evt| {
                            state.write().opt_gender = evt.value() == "true";
                        }
                    }
                    "성비 균형"
                }
            }

            div { class: "card",
                button {
                    class: "btn primary",
                    onclick: run_assign,
                    "🚀 반 배정 실행"
                }
                if !status_msg().is_empty() {
                    div {
                        style: "margin-top: 12px; color: #4b5563; text-align: center;",
                        "{status_msg}"
                    }
                }
            }
        }

        if let Some(classes) = state.read().assignments.clone() {
            AssignResult { classes }
        }
    }
}

#[component]
fn AssignResult(classes: Vec<ClassInfo>) -> Element {
    let total: usize = classes.iter().map(|c| c.students.len()).sum();
    let overall_avg: f32 = if total > 0 {
        classes
            .iter()
            .flat_map(|c| c.students.iter())
            .map(|s| s.score)
            .sum::<f32>()
            / total as f32
    } else {
        0.0
    };

    rsx! {
        div {
            style: "max-width: 1400px; margin: 32px auto 0 auto;",
            div {
                style: "display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 4px; gap: 8px; flex-wrap: wrap;",
                h2 { style: "margin: 0;", "배정 결과 요약" }
                div {
                    class: "no-print",
                    style: "display: flex; gap: 8px;",
                    button {
                        onclick: move |_| { let _ = document::eval("window.print();"); },
                        style: "padding: 6px 12px; background: #6c757d; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 14px;",
                        "🖨 인쇄"
                    }
                    Link {
                        to: Route::ResultDetail {},
                        style: "padding: 6px 12px; background: #2563eb; color: white; border-radius: 6px; text-decoration: none; font-size: 14px;",
                        "📋 상세 보기 →"
                    }
                }
            }
            p { style: "margin: 0 0 16px 0; color: #4b5563;",
                "총 {total}명 · 전체 평균 {overall_avg:.2}점"
            }
            div {
                style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 16px; align-items: start;",
                for c in classes.into_iter() {
                    ClassCard { key: "{c.id}", info: c }
                }
            }
        }
    }
}

#[component]
fn ResultDetail() -> Element {
    let state = use_context::<Signal<AppState>>();
    let maybe_classes = state.read().assignments.clone();

    rsx! {
        div {
            style: "max-width: 1200px; margin: 0 auto;",
            div {
                style: "display: flex; justify-content: space-between; align-items: baseline; gap: 8px; flex-wrap: wrap;",
                h1 { style: "margin: 0;", "배정 결과 상세" }
                if let Some(classes) = &maybe_classes {
                    {
                        let export_url = csv_data_url(&assignments_to_csv(classes));
                        rsx! {
                            div {
                                class: "no-print",
                                style: "display: flex; gap: 8px;",
                                button {
                                    onclick: move |_| { let _ = document::eval("window.print();"); },
                                    style: "padding: 8px 15px; background: #6c757d; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                                    "🖨 인쇄"
                                }
                                a {
                                    href: "{export_url}",
                                    download: "class-assignment.csv",
                                    style: "padding: 8px 15px; background: #6c757d; color: white; border-radius: 4px; text-decoration: none; font-size: 14px;",
                                    "📤 CSV 내보내기"
                                }
                            }
                        }
                    }
                }
            }

            if let Some(classes) = maybe_classes {
                {
                    let total: usize = classes.iter().map(|c| c.students.len()).sum();
                    let overall_avg: f32 = if total > 0 {
                        classes.iter().flat_map(|c| c.students.iter()).map(|s| s.score).sum::<f32>() / total as f32
                    } else { 0.0 };
                    rsx! {
                        p { style: "color: #4b5563; margin-top: 8px;", "총 {total}명 · 전체 평균 {overall_avg:.2}점" }
                    }
                }
                for c in classes.into_iter() {
                    ClassTable { key: "{c.id}", info: c }
                }
            } else {
                div {
                    style: "padding: 40px; background: #fff; border-radius: 8px; text-align: center; color: #6b7280;",
                    p { "아직 배정이 실행되지 않았습니다." }
                    p {
                        Link {
                            to: Route::AssignClass {},
                            style: "color: #2563eb; text-decoration: none;",
                            "→ 반 배정 페이지로 이동"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ClassTable(info: ClassInfo) -> Element {
    let avg = info.avg_score();
    let male = info.count_gender(Gender::Male);
    let female = info.count_gender(Gender::Female);
    let n = info.students.len();
    let id = info.id;

    rsx! {
        div { class: "print-class", style: "margin-bottom: 28px;",
            div {
                style: "display: flex; justify-content: space-between; align-items: baseline; padding: 8px 0;",
                h2 { style: "margin: 0;", "{id}반" }
                span {
                    style: "color: #4b5563; font-size: 14px;",
                    "{n}명 · 평균 {avg:.1}점 · 남 {male} / 여 {female}"
                }
            }
            table {
                style: "width: 100%; border-collapse: collapse; box-shadow: 0 1px 4px rgba(0,0,0,0.06); background: #fff;",
                thead {
                    tr { style: "background-color: #f8f9fa; text-align: left;",
                        th { style: "padding: 10px; border-bottom: 2px solid #dee2e6; width: 60px;", "ID" }
                        th { style: "padding: 10px; border-bottom: 2px solid #dee2e6;", "이름" }
                        th { style: "padding: 10px; border-bottom: 2px solid #dee2e6; width: 80px;", "성별" }
                        th { style: "padding: 10px; border-bottom: 2px solid #dee2e6; width: 80px;", "점수" }
                        th { style: "padding: 10px; border-bottom: 2px solid #dee2e6;", "비고" }
                    }
                }
                tbody {
                    for s in info.students.iter().cloned() {
                        ResultRow { key: "{s.id}", student: s }
                    }
                }
            }
        }
    }
}

#[component]
fn ResultRow(student: Student) -> Element {
    let name = student
        .name
        .clone()
        .unwrap_or_else(|| format!("(ID {})", student.id));
    let gender = student.gender.to_label();
    let gender_color = match student.gender {
        Gender::Male => "#2563eb",
        Gender::Female => "#db2777",
    };
    let score = student.score;
    let note = student.note.clone().unwrap_or_default();
    let id = student.id;

    rsx! {
        tr { style: "border-bottom: 1px solid #eee;",
            td { style: "padding: 8px; text-align: center; color: #6b7280;", "{id}" }
            td { style: "padding: 8px;", "{name}" }
            td { style: "padding: 8px; color: {gender_color}; font-weight: 600;", "{gender}" }
            td { style: "padding: 8px; text-align: right;", "{score:.1}" }
            td { style: "padding: 8px; color: #6b7280;", "{note}" }
        }
    }
}

#[component]
fn ClassCard(info: ClassInfo) -> Element {
    let avg = info.avg_score();
    let male = info.count_gender(Gender::Male);
    let female = info.count_gender(Gender::Female);
    let n = info.students.len();
    let id = info.id;

    rsx! {
        div {
            class: "print-class",
            style: "border: 1px solid #e5e7eb; border-radius: 8px; padding: 14px; background: #fff; box-shadow: 0 1px 2px rgba(0,0,0,0.04);",
            div {
                style: "display: flex; justify-content: space-between; align-items: baseline; border-bottom: 1px solid #f3f4f6; padding-bottom: 8px;",
                h3 { style: "margin: 0; font-size: 18px;", "{id}반" }
                span { style: "color: #6b7280; font-size: 13px;", "{n}명" }
            }
            div {
                style: "margin-top: 8px; font-size: 13px; color: #374151;",
                "평균 {avg:.1}점 · 남 {male} / 여 {female}"
            }
            div {
                style: "margin-top: 10px; display: grid; grid-template-columns: repeat(auto-fill, minmax(130px, 1fr)); gap: 4px 10px; font-size: 13px;",
                for s in info.students.iter().cloned() {
                    StudentRow { key: "{s.id}", student: s }
                }
            }
        }
    }
}

#[component]
fn StudentRow(student: Student) -> Element {
    let name = student
        .name
        .clone()
        .unwrap_or_else(|| format!("#{}", student.id));
    let gender = student.gender.to_label();
    let gender_color = match student.gender {
        Gender::Male => "#2563eb",
        Gender::Female => "#db2777",
    };
    let score = student.score;
    rsx! {
        div {
            style: "display: flex; justify-content: space-between; gap: 6px; padding: 2px 0; border-bottom: 1px dashed #f3f4f6;",
            span {
                style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                span { style: "color: {gender_color}; margin-right: 4px;", "{gender}" }
                "{name}"
            }
            span { style: "color: #6b7280;", "{score:.1}" }
        }
    }
}

#[component]
fn InfoPage() -> Element {
    let th_style =
        "text-align: left; padding: 8px; border-bottom: 2px solid #dee2e6; background: #f8f9fa;";
    let td_style = "padding: 6px 8px; border-bottom: 1px solid #eee; vertical-align: top;";
    let label_style =
        "padding: 4px 16px 4px 0; color: #6b7280; font-weight: 600; white-space: nowrap;";
    let section_style = "margin-top: 24px;";

    rsx! {
        div { style: "max-width: 860px; margin: 0 auto;",
            h1 { "프로그램 정보" }

            h2 { style: "{section_style}", "개요" }
            p {
                "학교 반 배정을 도와주는 웹 애플리케이션입니다. 학생 명단(이름·성별·점수·비고)을 입력하거나 CSV 로 불러온 뒤, 평균 점수와 성비의 균형을 맞춰 여러 학급으로 자동 배정합니다."
            }

            h2 { style: "{section_style}", "기본 정보" }
            table { style: "border-collapse: collapse;",
                tbody {
                    tr {
                        td { style: "{label_style}", "프로그램명" }
                        td { style: "padding: 4px 0;", "class-assigner-dx" }
                    }
                    tr {
                        td { style: "{label_style}", "버전" }
                        td { style: "padding: 4px 0;", "0.1.0" }
                    }
                    tr {
                        td { style: "{label_style}", "저작자" }
                        td { style: "padding: 4px 0;", "wdpk39 <wdpk39@gmail.com>" }
                    }
                }
            }

            h2 { style: "{section_style}", "기술 스택" }
            ul {
                li { b { "언어: " } "Rust (edition 2021)" }
                li { b { "UI 프레임워크: " } "Dioxus 0.7" }
                li { b { "실행 환경: " } "브라우저 WebAssembly (target: wasm32-unknown-unknown)" }
                li { b { "스타일: " } "CSS (Tailwind 자동 모드)" }
                li { b { "배포: " } "정적 파일 (서버리스)" }
            }

            h2 { style: "{section_style}", "사용 라이브러리" }
            table {
                style: "width: 100%; border-collapse: collapse; font-size: 14px;",
                thead {
                    tr {
                        th { style: "{th_style}", "이름" }
                        th { style: "{th_style}", "버전" }
                        th { style: "{th_style}", "용도" }
                        th { style: "{th_style}", "라이센스" }
                    }
                }
                tbody {
                    tr {
                        td { style: "{td_style} font-family: monospace;", "dioxus" }
                        td { style: "{td_style} color: #6b7280;", "0.7" }
                        td { style: "{td_style}", "UI / 라우터" }
                        td { style: "{td_style} color: #6b7280;", "MIT OR Apache-2.0" }
                    }
                    tr {
                        td { style: "{td_style} font-family: monospace;", "dioxus-logger" }
                        td { style: "{td_style} color: #6b7280;", "0.7" }
                        td { style: "{td_style}", "로깅 초기화" }
                        td { style: "{td_style} color: #6b7280;", "MIT OR Apache-2.0" }
                    }
                    tr {
                        td { style: "{td_style} font-family: monospace;", "log" }
                        td { style: "{td_style} color: #6b7280;", "0.4" }
                        td { style: "{td_style}", "로그 파사드" }
                        td { style: "{td_style} color: #6b7280;", "MIT OR Apache-2.0" }
                    }
                    tr {
                        td { style: "{td_style} font-family: monospace;", "rand" }
                        td { style: "{td_style} color: #6b7280;", "0.9" }
                        td { style: "{td_style}", "난수 생성 / 셔플" }
                        td { style: "{td_style} color: #6b7280;", "MIT OR Apache-2.0" }
                    }
                    tr {
                        td { style: "{td_style} font-family: monospace;", "rand_distr" }
                        td { style: "{td_style} color: #6b7280;", "0.5" }
                        td { style: "{td_style}", "샘플 데이터용 정규분포" }
                        td { style: "{td_style} color: #6b7280;", "MIT OR Apache-2.0" }
                    }
                    tr {
                        td { style: "{td_style} font-family: monospace;", "getrandom" }
                        td { style: "{td_style} color: #6b7280;", "0.3" }
                        td { style: "{td_style}", "WASM 난수 소스 (wasm_js)" }
                        td { style: "{td_style} color: #6b7280;", "MIT OR Apache-2.0" }
                    }
                    tr {
                        td { style: "{td_style} font-family: monospace;", "gloo-timers" }
                        td { style: "{td_style} color: #6b7280;", "0.3" }
                        td { style: "{td_style}", "비동기 타이머" }
                        td { style: "{td_style} color: #6b7280;", "MIT OR Apache-2.0" }
                    }
                }
            }
            p { style: "font-size: 13px; color: #6b7280; margin-top: 8px;",
                "각 라이브러리의 저작권 및 라이센스 원문은 해당 프로젝트의 저장소를 따릅니다."
            }

            h2 { style: "{section_style}", "외부 리소스" }
            ul {
                li { "Google Fonts — Noto Color Emoji (이모지 렌더링, SIL Open Font License 1.1)" }
            }

            h2 { style: "{section_style}", "프로그램 라이센스" }
            p {
                "본 프로그램은 "
                b { "MIT License" }
                " 하에 배포됩니다. 자유롭게 사용·수정·재배포할 수 있으며, 사용으로 인한 결과에 대해 저작자는 책임을 지지 않습니다. 전문은 저장소의 "
                code { "LICENSE" }
                " 파일을 참조하세요."
            }

            h2 { style: "{section_style}", "개발 도구 (LLM)" }
            p {
                "본 프로그램의 설계·구현 과정에서 대규모 언어 모델(LLM)을 보조 도구로 활용했습니다."
            }
            ul {
                li { b { "Claude" } " (Anthropic) — 코드 생성, 리팩터링, 버그 분석" }
                li { "그 외 LLM 기반 코드 어시스턴트" }
            }
            p { style: "font-size: 13px; color: #6b7280;",
                "최종 코드의 구조·동작 검증 및 책임은 저작자에게 있습니다."
            }

            h2 { style: "{section_style}", "데이터 처리 방침" }
            ul {
                li { "모든 연산은 접속한 장치의 브라우저 내에서만 수행됩니다." }
                li { "학생 정보를 서버로 전송하거나 외부에 저장하지 않습니다." }
                li { "새로고침하거나 탭을 닫으면 입력한 데이터가 사라집니다." }
                li { "영구 보관이 필요하면 학생 목록 페이지에서 CSV 내보내기를 사용하세요." }
            }
        }
    }
}

// ★ Egui를 보여줄 컴포넌트
#[component]
fn EguiPage() -> Element {
    rsx! {
        div { class: "egui-container",
            h1 { "Egui Integration" }
            p { "아래 영역은 WASM으로 컴파일된 별도의 Egui 애플리케이션입니다." }

            // iframe을 통해 로컬 assets에 있는 egui 앱을 로드
            iframe {
                src: "/egui-app/application.html", // assets 폴더 경로 (Dioxus 설정에 따라 /egui-app/index.html 일 수도 있음)
                class: "egui-frame",
                title: "Egui Application"
            }
        }
    }
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        h1 { "페이지를 찾을 수 없습니다." }
    }
}
