use maud::{html, Markup, DOCTYPE};

pub fn header(title: &str) -> Markup {
    html! {
        h1 {
            (title)
        }
    }
}

pub fn page(title: &str, logout: bool, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { (title) " — EWU Timetable" }
                meta name="viewport" content="width=device-width, initial-scale=1";
                link rel="stylesheet" type="text/css" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.fluid.classless.min.css";
            }
            body {
                header {
                    nav {
                        ul {
                            li { strong { a href="/" { "EWU Timetable" } } }
                        }
                        ul {
                            li { a href="https://github.com/arafatamim/ewubd-timetable" { "Source code" } }
                            @if logout {
                                li { a href="/logout" { "Logout" } }
                            }
                        }
                    }
                    (header(title))
                }
                main {
                    (body)
                }
            }
        }
    }
}
