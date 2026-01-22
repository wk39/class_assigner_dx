#![allow(non_snake_case)]

use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture; // 비동기 지연(sleep)용
                                        //
use rand::{rngs::StdRng, Rng as _, SeedableRng as _};
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
    // 화면 표시용 텍스트
    // fn to_label(&self) -> &'static str {
    //     match self {
    //         Gender::Male => "남",
    //         Gender::Female => "여",
    //     }
    // }
    // HTML value 속성용
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

#[derive(Debug, Clone)]
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
    // username: &'static str,
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
            next_student_id: 4,

            //
            opt_score: true,
            opt_gender: true,
            //
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

// ★ 핵심 기능: 무거운 작업과 프로그레스 바
#[component]
fn AssignClass() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    // Signals: 상태 관리
    let mut is_running = use_signal(|| false); // 작업 실행 중 여부
    let mut progress = use_signal(|| 0); // 진행률 (0 ~ 100)

    // 작업 시작 핸들러
    let start_processing = move |_| {
        if is_running() {
            return;
        } // 중복 실행 방지

        // 상태 초기화
        is_running.set(true);
        progress.set(0);

        // 비동기 작업 스폰 (spawn)
        // 메인 스레드(UI)를 차단하지 않기 위해 spawn을 사용합니다.
        spawn(async move {
            for i in 1..=100 {
                // 무거운 작업을 흉내내기 위해 10ms 대기 (실제 로직 대체 가능)
                TimeoutFuture::new(10).await;

                // 진행률 업데이트 -> UI 자동 렌더링
                progress.set(i);
            }
            // 완료 후 상태 복귀
            is_running.set(false);
        });
    };

    rsx! {
        div { class: "task-container",
            h1 { "반 배정" }
            h2 { "학급 수를 설정합니다." }

            // 3. 슬라이드 바 구현
            input {
                style: "margin-top: 10px; margin-left: 20px;",
                r#type: "range", // 슬라이더 타입
                min: "3",        // 최소값
                max: "30",      // 최대값
                step: "1",       // 이동 단위

                // [중요] 현재 상태를 슬라이더 위치에 반영 (Two-way binding의 절반)
                value: "{state().number_of_class}",

                // [중요] 슬라이더 움직임 감지하여 상태 업데이트
                oninput: move |evt| {
                    // 입력값은 문자열로 들어오므로 숫자로 변환
                    if let Ok(val) = evt.value().parse::<u8>() {
                        state.write().number_of_class = val;
                    }
                }
            }

            // 현재 슬라이더 값 옆에 표시
            span {
                style: "margin-left: 10px; font-weight: bold;",
                "{state().number_of_class} 반"
            }

            h2 { "최적화 기준을 선택합니다." }
                // [체크박스 구현 부분]
                div {
                    style: "margin-top: 10px; margin-left: 20px;",
                    label {
                        // 클릭 영역을 넓히기 위해 label 안에 input을 넣는 패턴 권장
                        style: "cursor: pointer; display: flex; align-items: center;",

                        input {
                            r#type: "checkbox",
                            style: "width: 20px; height: 20px; margin-right: 8px;", // 크기 키우기

                            // 1. 현재 상태 반영 (true면 체크표시)
                            checked: "{state().opt_score}",

                            // 2. 클릭 시 상태 변경
                            oninput: move |evt| {
                                // Dioxus에서 checkbox의 evt.value()는 "true" 또는 "false" 문자열을 반환함
                                let is_checked = evt.value() == "true";
                                state.write().opt_score = is_checked;
                            }
                        }
                        "평균점수 균형"
                    }

                    label {
                        // 클릭 영역을 넓히기 위해 label 안에 input을 넣는 패턴 권장
                        style: "cursor: pointer; display: flex; align-items: center;",

                        input {
                            r#type: "checkbox",
                            style: "width: 20px; height: 20px; margin-right: 8px;", // 크기 키우기

                            // 1. 현재 상태 반영 (true면 체크표시)
                            checked: "{state().opt_gender}",

                            // 2. 클릭 시 상태 변경
                            oninput: move |evt| {
                                // Dioxus에서 checkbox의 evt.value()는 "true" 또는 "false" 문자열을 반환함
                                let is_checked = evt.value() == "true";
                                state.write().opt_gender = is_checked;
                            }
                        }
                        "성비 균형"
                    }
                }

            h2 { "알고리즘을 선택합니다." }
            p { ">> 🚧 Under Construction" }

            div { class: "card",
                // 프로그레스 바 상단 텍스트
                div { class: "progress-info",
                    span {
                        "Status: " // 일반 텍스트
                        if is_running() { "처리 중..." } else { "대기 중" } // Rust 코드 블록
                    }
                    span { "{progress}%" }
                }


                // HTML5 Progress Bar
                progress {
                    class: "styled-progress",
                    value: "{progress}",
                    max: "100"
                }

                // 실행 버튼
                button {
                    class: if is_running() { "btn disabled" } else { "btn primary" },
                    disabled: "{is_running}", // 실행 중이면 클릭 불가
                    onclick: start_processing,

                    if is_running() {
                        "⏳ 작업 수행 중..."
                    } else {
                        "🚀 작업 시작"
                    }
                }

                // 완료 메시지
                if progress() == 100 && !is_running() {
                    div { class: "success-message", "✅ 모든 작업이 완료되었습니다!" }
                }
            }
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
