async function get_latest() {
    const raw = await fetch("/issues");
    const plaintext = await raw.text();

    document.getElementById("latest-div").innerHTML = plaintext;
}

get_latest();
