/* Keep the navigation destination visible without moving the document or focus. */
document.addEventListener("DOMContentLoaded", () => {
  const sidebar = document.querySelector(".sidebar-tree");
  const scroller = sidebar?.closest(".sidebar-scroll");
  if (!sidebar || !scroller) return;

  const links = [...sidebar.querySelectorAll("a[href]")];
  const storageKey = `sidebar-scroll:${links[0].pathname}`;
  try {
    const saved = sessionStorage.getItem(storageKey);
    if (saved !== null) scroller.scrollTop = Number(saved);
  } catch {
    // Navigation also works when browser storage is unavailable.
  }
  function savePosition() {
    try {
      sessionStorage.setItem(storageKey, String(scroller.scrollTop));
    } catch {
      // Storage can be disabled in private or local-file browsing.
    }
  }
  window.addEventListener("pagehide", savePosition);

  const pageLinks = links.filter((link) =>
    link.origin === location.origin && link.pathname === location.pathname
  );
  const pageLink = pageLinks.find((link) => !link.hash);
  const sectionLinks = new Map();
  for (const link of pageLinks) {
    if (link.hash && link.hash !== "#next-steps") {
      sectionLinks.set(decodeURIComponent(link.hash.slice(1)), link);
    }
  }
  let activeLink = sidebar.querySelector("[aria-current]");

  function keepVisible() {
    if (!activeLink || !scroller.clientHeight) return;
    const item = activeLink.getBoundingClientRect();
    const viewport = scroller.getBoundingClientRect();
    const margin = 16;
    if (item.top < viewport.top + margin || item.bottom > viewport.bottom - margin) {
      scroller.scrollTo({
        top: scroller.scrollTop + item.top - viewport.top
          - (scroller.clientHeight - item.height) / 2,
        behavior: "instant",
      });
    }
  }

  function update() {
    let target;
    try {
      target = document.getElementById(decodeURIComponent(location.hash.slice(1)));
    } catch {
      // A hand-written URL may contain an invalid percent escape.
    }
    // Nested headings use their nearest section represented in the sidebar.
    while (target && !sectionLinks.has(target.id)) target = target.parentElement;
    const link = (target && sectionLinks.get(target.id)) || pageLink;
    if (link !== activeLink) {
      activeLink?.removeAttribute("aria-current");
      link?.setAttribute("aria-current", link === pageLink ? "page" : "location");
      activeLink = link;
    }
    keepVisible();
  }

  window.addEventListener("hashchange", update);
  window.addEventListener("pageshow", (event) => {
    if (event.persisted) update();
  });
  window.addEventListener("resize", keepVisible);
  document.querySelector('input[name="__navigation"]')?.addEventListener("change", () => {
    requestAnimationFrame(keepVisible);
  });
  update();
  document.fonts.ready.then(keepVisible);
});
