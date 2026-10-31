/* Follow the visible section without moving the document or keyboard focus. */
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
  const sections = pageLinks
    .filter((link) => link.hash && link.hash !== "#next-steps")
    .map((link) => ({
      link,
      element: document.getElementById(decodeURIComponent(link.hash.slice(1))),
      top: 0,
    }));
  let activeLink = sidebar.querySelector("[aria-current]");

  function keepVisible(link = activeLink, container = scroller) {
    if (!link || !container.clientHeight) return;
    const item = link.getBoundingClientRect();
    const viewport = container.getBoundingClientRect();
    const margin = 16;
    if (item.top < viewport.top + margin || item.bottom > viewport.bottom - margin) {
      container.scrollTo({
        top: container.scrollTop + item.top - viewport.top
          - (container.clientHeight - item.height) / 2,
        behavior: "instant",
      });
    }
  }

  function update() {
    // Treat a heading near the top of the viewport as the current topic.
    const readingPosition = window.scrollY + Math.min(120, window.innerHeight / 4);
    let link = pageLink;
    for (const section of sections) {
      if (section.top > readingPosition) break;
      link = section.link;
    }
    if (link === activeLink) return;
    activeLink?.removeAttribute("aria-current");
    link?.setAttribute("aria-current", link === pageLink ? "page" : "location");
    activeLink = link;
    keepVisible();
  }

  function measure() {
    for (const section of sections) {
      section.top = section.element.getBoundingClientRect().top + window.scrollY;
    }
    update();
    keepVisible();
  }

  let pending = false;
  function scheduleUpdate() {
    if (pending) return;
    pending = true;
    requestAnimationFrame(() => {
      pending = false;
      update();
    });
  }

  // Scroll updates use cached heading positions and only change two links.
  window.addEventListener("scroll", scheduleUpdate, { passive: true });
  window.addEventListener("hashchange", scheduleUpdate);
  window.addEventListener("pageshow", measure);
  window.addEventListener("resize", measure);
  document.querySelector('input[name="__navigation"]')?.addEventListener("change", () => {
    requestAnimationFrame(() => keepVisible());
  });
  // Images and other content can shift headings after the initial layout.
  const article = document.querySelector("article");
  if (article) new ResizeObserver(measure).observe(article);
  // Furo already tracks TOC sections; reuse its events rather than a second observer.
  const toc = document.querySelector(".toc-tree");
  const tocScroller = toc?.closest(".toc-scroll");
  if (tocScroller) {
    let activeTocLink;
    function updateToc(link) {
      activeTocLink?.removeAttribute("aria-current");
      link?.setAttribute("aria-current", "location");
      activeTocLink = link;
      if (link) keepVisible(link, tocScroller);
    }
    toc.addEventListener("gumshoeActivate", (event) => updateToc(event.detail.link));
    toc.addEventListener("gumshoeDeactivate", () => {
      activeTocLink?.removeAttribute("aria-current");
      activeTocLink = null;
    });
    requestAnimationFrame(() => updateToc(toc.querySelector(".scroll-current > a")));
    document.querySelector('input[name="__toc"]')?.addEventListener("change", () => {
      requestAnimationFrame(() => {
        if (activeTocLink) keepVisible(activeTocLink, tocScroller);
      });
    });
  }
  measure();
  document.fonts.ready.then(measure);
});
