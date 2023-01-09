pub mod verify;

use http::Uri;

pub enum Handled {
    Html(String),
    Redirect(Uri),
}
