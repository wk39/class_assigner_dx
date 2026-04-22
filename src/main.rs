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

#[derive(Clone)]
struct AppState {
    count: i32,
    number_of_class: u8,
    //
    students: Vec<Student>,
    next_student_id: u32,
    //
    opt_score: bool,
    opt_gender: bool,
    //
    assignments: Option<Vec<ClassInfo>>,
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
            count: 0,
            number_of_class: 10,
            //
            students,
            next_student_id: n_students,

            //
            opt_score: true,
            opt_gender: true,
            //
            assignments: None,
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
                    li { Link { to: Route::InfoPage {}, class: "nav-item", "💁 정보" } }
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
fn MainPage() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    rsx! {
        div { class : "task-container",
            h1 { "🚧 Under Construction" }
            p { "간략한 학교 반 배정 프로그램입니다." }
            p { "프로그램 내 모든 작업은 접속한 장치(PC, Mobile Device 등)에서 수행되며, 서버로 데이터를 전송하지 않습니다." }
            p { "따라서 작업 내용을 저장하지 않으면 데이터를 잃을 수 있으니 주의해야 합니다." }
            b { "작업 순서 :" }
            ol {
                li { "학생 목록을 구성합니다"}
                li { "반 배정을 수행합니다" }
            }

            // 상태 변경
            button {
                onclick: move |_| state.write().count += 1,
                "Increment"
            }

        }
    }
}

fn StudentList() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    // 학생 추가
    let add_student = move |_| {
        let mut ss = state.write();
        let id = ss.next_student_id;
        ss.students.push(Student::new(id, None, Gender::Male, 0.0));
        ss.next_student_id += 1;
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
    rsx! {
        div {
            style: "max-width: 900px;",

            // 상단 버튼
            div { style: "text-align: right; margin-bottom: 10px;",
                button {
                    onclick: add_student,
                    style: "padding: 8px 15px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    "+ 학생 추가"
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
                            style: "border-bottom: 1px solid #eee; height: 50px;", // 행 높이 지정

                            // 1. ID (읽기 전용)
                            td { style: "padding: 10px; text-align: center; color: #666;", "{student.id}" }

                            // 2. 이름 (Option<String>)
                            td { style: "padding: 10px;",
                                input {
                                    r#type: "text",
                                    style: "width: 100%; padding: 5px; border: 1px solid #ccc; border-radius: 4px;",
                                    placeholder: "이름 입력",
                                    // Some이면 값, None이면 빈 문자열
                                    value: "{student.name.clone().unwrap_or_default()}",
                                    oninput: move |evt| update_name(student.id, evt.value())
                                }
                            }

                            // 3. 성별 (Enum -> Select Box)
                            td { style: "padding: 10px;",
                                select {
                                    style: "width: 100%; padding: 5px; border: 1px solid #ccc; border-radius: 4px;",
                                    value: "{student.gender.to_value()}", // 현재 값 선택
                                    oninput: move |evt| update_gender(student.id, evt.value()),

                                    option { value: "male", "남" }
                                    option { value: "female", "여" }
                                }
                            }

                            // 4. 점수 (f32)
                            td { style: "padding: 10px;",
                                input {
                                    r#type: "number",
                                    // step="0.1"을 주어야 소수점 입력 가능
                                    step: "0.1",
                                    style: "width: 100%; padding: 5px; border: 1px solid #ccc; border-radius: 4px; text-align: right;",
                                    value: "{student.score}",
                                    oninput: move |evt| update_score(student.id, evt.value())
                                }
                            }

                            // 5. 비고 (Option<String>)
                            td { style: "padding: 10px;",
                                input {
                                    r#type: "text",
                                    style: "width: 100%; padding: 5px; border: 1px solid #ccc; border-radius: 4px;",
                                    placeholder: "특이사항 없음",
                                    value: "{student.note.clone().unwrap_or_default()}",
                                    oninput: move |evt| update_note(student.id, evt.value())
                                }
                            }

                            // 6. 삭제 버튼
                            td { style: "padding: 10px; text-align: center;",
                                button {
                                    onclick: move |_| remove_student(student.id),
                                    style: "background: #dc3545; color: white; border: none; border-radius: 4px; padding: 5px 10px; cursor: pointer;",
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
            (s.students.clone(), s.number_of_class, s.opt_score, s.opt_gender)
        };
        if students.is_empty() {
            status_msg.set("학생 목록이 비어 있습니다.".to_string());
            state.write().assignments = None;
            return;
        }
        let result = assign_classes(&students, k, opt_s, opt_g);
        status_msg.set(format!("{}명을 {}개 반으로 배정했습니다.", students.len(), k));
        state.write().assignments = Some(result);
    };

    rsx! {
        div { class: "task-container",
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
                style: "display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 4px;",
                h2 { style: "margin: 0;", "배정 결과 요약" }
                Link {
                    to: Route::ResultDetail {},
                    style: "padding: 6px 12px; background: #2563eb; color: white; border-radius: 6px; text-decoration: none; font-size: 14px;",
                    "📋 상세 보기 →"
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
            h1 { "배정 결과 상세" }

            if let Some(classes) = maybe_classes {
                {
                    let total: usize = classes.iter().map(|c| c.students.len()).sum();
                    let overall_avg: f32 = if total > 0 {
                        classes.iter().flat_map(|c| c.students.iter()).map(|s| s.score).sum::<f32>() / total as f32
                    } else { 0.0 };
                    rsx! {
                        p { style: "color: #4b5563;", "총 {total}명 · 전체 평균 {overall_avg:.2}점" }
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
        div { style: "margin-bottom: 28px;",
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
    let score = student.score;
    let note = student.note.clone().unwrap_or_default();
    let id = student.id;

    rsx! {
        tr { style: "border-bottom: 1px solid #eee;",
            td { style: "padding: 8px; text-align: center; color: #6b7280;", "{id}" }
            td { style: "padding: 8px;", "{name}" }
            td { style: "padding: 8px;", "{gender}" }
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

// ★ 핵심 기능: 무거운 작업과 프로그레스 바
#[component]
fn InfoPage() -> Element {
    rsx! {
        div { class: "task-container",
            h1 { "프로그램 정보" }
            p { "버튼을 누르면 비동기 작업이 시작됩니다." }
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
