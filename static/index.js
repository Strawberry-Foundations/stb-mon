async function onLogin() {
    console.log();
    let password = document.getElementById("password");
    let res = await fetch(`/api/create_session?pw=${password.value}`, { method: "POST" });
    if (res.status == 401) {
        password.value = "";
        password.placeholder = "wrong password";
        return;
    }
    
    let token = await res.text();
    let date = new Date();
    let now = date.getTime();
    let expiry = now + 60 * 60 * 24 * 7; // 7 days
    date.setTime(expiry);
    document.cookie = `token=${token}; expiry=${date.toUTCString()}`;
    document.location.href = "/admin"
}