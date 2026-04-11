pub fn sanitise_pings(message: &str) -> String {
    message.replace('@', "@\u{200B}")
}

pub fn validate_token(token: Option<&str>) -> Result<&str, &str> {
    let Some(token) = token else {
        return Err("no token was provided");
    };

    let token = match serenity::utils::validate_token(token) {
        Ok(()) => token,
        Err(_) => return Err("an invalid token was provided"),
    };

    Ok(token)
}
