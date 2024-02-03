document.body.addEventListener("htmx:beforeSend", function(evt) {
    console.log("Got htmx conf req");
    console.log(evt.detail.elt);

    if (evt.detail.requestConfig.verb === "post" && evt.detail.elt["htmx-internal-data"].path === "/new/comment") {
        console.log(document.getElementById("issue-id").innerHTML);
        evt.detail.requestConfig.parameters["issue_id"] = document.getElementById("issue-id").innerHTML;
    }
})
