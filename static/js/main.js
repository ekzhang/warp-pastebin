hljs.listLanguages().forEach((lang) => {
  const option = document.createElement("option");
  option.innerText = lang;
  if (lang == "plaintext") option.selected = true;
  document.getElementById("lang").appendChild(option);
});

document.getElementById("form").addEventListener("submit", (event) => {
  event.preventDefault();
  const text = document.getElementById("text").value;
  const lang = document.getElementById("lang").value || "plaintext";
  superagent
    .post(`/api/paste`)
    .send({ text, lang })
    .then((resp) => {
      history.pushState({}, "", `#${resp.body.id}`);
      loadPaste(resp.body.id);
    })
    .catch((error) => {
      alert(error);
    });
});

async function loadPaste(id) {
  try {
    const animateOut = anime({
      targets: "#input-col",
      opacity: 0,
      easing: "easeInCubic",
      duration: 500,
    }).finished;
    const { body } = await superagent.get(`/api/paste/${id}`);
    const el = document.getElementById("output");
    el.innerHTML = hljs.highlight(body.lang, body.text, true).value;
    await animateOut;
    document.getElementById("input-col").style.display = "none";
    document.getElementById("output-col").style.display = "block";
    anime({
      targets: "#output-col",
      opacity: 1,
      easing: "easeOutCubic",
      duration: 500,
    }).finished;
    anime({
      targets: "body",
      background: "#161616",
      easing: "easeOutCubic",
      duration: 500,
    }).finished;
  } catch (error) {
    location.href = "/";
  }
}

if (location.hash) {
  loadPaste(location.hash.substr(1));
} else {
  document.getElementById("input-col").style.display = "block";
}
