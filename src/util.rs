pub fn sanitise_pings(message: &str) -> String {
    message.replace("@", "@​")
}

pub fn validate_token(token: Option<&str>) -> Result<&str, &str> {
    let token = match token {
        Some(token) => token,
        None => return Err("no token was provided")
    };

    let token = match serenity::utils::validate_token(token) {
        Ok(_) => token,
        Err(_) => return Err("an invalid token was provided")
    };

    Ok(token)
}