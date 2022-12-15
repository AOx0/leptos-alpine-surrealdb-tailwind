use super::*;

#[derive(Deserialize)]
pub struct Data {
    email: String,
    pass: String,
}

pub async fn login(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    result: Result<Json<Data>, JsonRejection>,
) -> Result<(PrivateCookieJar, Redirect), String> {
    let payload = if let Err(error) = result {
        return Err(format!("{}", error));
    } else {
        result.unwrap()
    };

    if payload.email.trim().is_empty() {
        return Err("Email must have a value".to_owned());
    }

    if payload.pass.trim().is_empty() {
        return Err("Password must have a value".to_owned());
    }

    let query_result = state
        .sql1_expect1(format!(
            "SELECT * FROM users WHERE email = '{}' AND crypto::argon2::compare(pass, '{}')",
            payload.email, payload.pass
        ))
        .await;

    let response = if query_result.is_err() {
        let msg = query_result.unwrap_err().to_string();
        if msg == "Failed to get first Object of query response" {
            return Err("There is no email/password match".to_owned());
        } else {
            return Err("There was an error retreiving data from the db".to_owned());
        }
    } else {
        query_result.unwrap()
    };
    let uid = Uuid::new_v4();
    let query_result = state
        .sql1_expect1(format!(
            "CREATE sessions SET token = '{}', user = {}, ip = '{}', time = time::now()",
            uid,
            response.get("id").unwrap(),
            addr.ip()
        ))
        .await;

    if query_result.is_err() {
        return Err("Failed to communicate with the database".to_string());
    }

    // We remove any existing 'tok' cookie
    let jar = {
        let cookie = jar.get("tok");
        if cookie.is_some() {
            jar.remove(cookie.unwrap())
        } else {
            jar
        }
    };

    Ok((
        jar.add({
            let mut a = Cookie::new("tok", uid.to_string());
            a.set_same_site(SameSite::Strict);
            a.set_path("/");
            a
        }),
        Redirect::permanent("/hello"),
    ))
}
