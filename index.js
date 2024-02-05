
function setCookie(cname, cvalue, exdays) {
  const d = new Date();
  d.setTime(d.getTime() + (exdays * 24 * 60 * 60 * 1000));
  let expires = "expires="+d.toUTCString();
  document.cookie = cname + "=" + cvalue + ";" + expires + ";path=/";
}

function getCookie(cname) {
  let name = cname + "=";
  let ca = document.cookie.split(';');
  for(let i = 0; i < ca.length; i++) {
    let c = ca[i];
    while (c.charAt(0) == ' ') {
      c = c.substring(1);
    }
    if (c.indexOf(name) == 0) {
      return c.substring(name.length, c.length);
    }
  }
  return "";
}

function fetchNewToken() {
    const req = new XMLHttpRequest();
    req.open("POST", "/auth/new_tok", false);

    req.setRequestHeader("Content-Type", "application/x-www-form-urlencoded");

    const b64 = btoa(document.getElementById("passwd-field").value);
    const fixed = b64.replace(/=/g, "-");

    console.log(b64);

    req.send("user=" + document.getElementById("user-field").value + "&passwd=" + fixed);

    setCookie("user_jwt", req.responseText, 30);
}

document.body.addEventListener("htmx:beforeSend", function(evt) {
    console.log("Got htmx conf req");
    console.log(evt.detail.elt);

    evt.detail.requestConfig.parameters["jwt"] = getCookie("user_jwt");

    if (evt.detail.requestConfig.verb === "post" && evt.detail.elt["htmx-internal-data"].path === "/new/comment") {
        console.log(document.getElementById("issue-id").innerHTML);
        evt.detail.requestConfig.parameters["issue_id"] = document.getElementById("issue-id").innerHTML;
    }
});

document.body.addEventListener("htmx:responseError", function(evt) {
    console.warn("Got htmx response err");

    if (evt.detail.xhr.status === 403) {
        fetchNewToken();
    }
});
