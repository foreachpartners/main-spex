// Tooling shared by every mockup page, outside the design: its stylesheet and a link back to the mockup index.
const indexUrl = new URL("../mockups.html", document.currentScript.src).href;

const stylesheet = document.createElement("link");
stylesheet.rel = "stylesheet";
stylesheet.href = new URL("mockup.css", document.currentScript.src).href;
document.head.append(stylesheet);

document.addEventListener("DOMContentLoaded", () => {
  if (window.location.pathname === new URL(indexUrl).pathname) {
    return;
  }
  const toolbar = document.createElement("header");
  toolbar.className = "mockup__toolbar";
  const indexLink = document.createElement("a");
  indexLink.href = indexUrl;
  indexLink.textContent = "All mockups";
  toolbar.append(indexLink);
  document.body.prepend(toolbar);
});
