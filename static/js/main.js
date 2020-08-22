document.getElementById("form").addEventListener("submit", (event) => {
  event.preventDefault();
  const text = document.getElementById("exampleMessage").value;
  axios
    .post(`/api/paste`, { text })
    .then((resp) => {
      history.pushState({}, "", `#${resp.data.id}`);
      loadPaste(resp.data.id);
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
    const { data } = await axios.get(`/api/paste/${id}`);
    const el = document.getElementById("output");
    el.innerText = data;
    hljs.highlightBlock(el);
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
