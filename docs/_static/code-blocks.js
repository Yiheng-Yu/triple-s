/* Keep code copying independent of syntax highlighting and theme selection. */
document.addEventListener("DOMContentLoaded", () => {
  document.querySelectorAll("article .highlight").forEach((panel) => {
    const pre = panel.querySelector("pre");
    if (!pre) return;

    if (panel.closest(".code-output")) {
      const label = document.createElement("div");
      label.className = "code-output-label";
      label.textContent = "Output";
      panel.prepend(label);
    }

    const button = document.createElement("button");
    button.type = "button";
    button.className = "code-copy-button";
    button.innerHTML = '<svg viewBox="0 0 32 32" aria-hidden="true" focusable="false"><path d="M28 10v18H10V10h18m0-2H10a2 2 0 0 0-2 2v18a2 2 0 0 0 2 2h18a2 2 0 0 0 2-2V10a2 2 0 0 0-2-2Z"/><path d="M4 18H2V4a2 2 0 0 1 2-2h14v2H4Z"/></svg>';
    button.title = "Copy code";
    button.setAttribute("aria-label", "Copy code to clipboard");
    const status = document.createElement("span");
    status.className = "code-copy-status";
    status.setAttribute("role", "status");
    button.append(status);
    panel.append(button);
    let resetTimer;
    let text;

    button.addEventListener("click", async () => {
      if (text === undefined) {
        if (pre.querySelector(".linenos")) {
          const code = pre.cloneNode(true);
          code.querySelectorAll(".linenos").forEach((line) => line.remove());
          text = code.textContent;
        } else {
          text = pre.textContent;
        }
      }
      let copied = false;
      try {
        await navigator.clipboard.writeText(text);
        copied = true;
      } catch {
        // Support local HTML previews and browsers without Clipboard API access.
        const field = document.createElement("textarea");
        field.value = text;
        field.style.cssText = "position:fixed;left:-9999px;top:0";
        document.body.append(field);
        field.select();
        try {
          copied = document.execCommand("copy");
        } catch {
          copied = false;
        } finally {
          field.remove();
          button.focus({ preventScroll: true });
        }
      }
      status.textContent = copied ? "Copied" : "Copy failed";
      button.dataset.copied = String(copied);
      button.setAttribute("aria-label", copied ? "Code copied" : "Copy failed; select code and copy manually");
      clearTimeout(resetTimer);
      resetTimer = setTimeout(() => {
        status.textContent = "";
        delete button.dataset.copied;
        button.setAttribute("aria-label", "Copy code to clipboard");
      }, 2000);
    });
  });
});
