pub fn sanitise_pings(message: &str) -> String {
    message.replace("@", "@​")
}
