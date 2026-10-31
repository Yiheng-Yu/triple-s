/* Share Furo's section tracking between the global sidebar and page TOC. */
document.addEventListener("DOMContentLoaded", () => {
  const sidebar = document.querySelector(".sidebar-tree");
  const scroller = sidebar?.closest(".sidebar-scroll");
  if (!scroller) return;

  const links = [...sidebar.querySelectorAll("a[href]")];
  const storageKey = `sidebar-scroll:${links[0].pathname}`;
  try {
    const saved = sessionStorage.getItem(storageKey);
    if (saved !== null) scroller.scrollTop = Number(saved);
  } catch {
    // Storage may be unavailable in local previews or private browsing.
  }
  window.addEventListener("pagehide", () => {
    try {
      sessionStorage.setItem(storageKey, String(scroller.scrollTop));
    } catch {
      // Navigation does not depend on storage.
    }
  });

  const pageLinks = links.filter((link) =>
    link.origin === location.origin && link.pathname === location.pathname
  );
  const pageLink = pageLinks.find((link) => !link.hash);
  const sections = new Map(pageLinks
    .filter((link) => link.hash && link.hash !== "#next-steps")
    .map((link) => [link.hash, link]));
  const toc = document.querySelector(".toc-tree");
  const tocScroller = toc?.closest(".toc-scroll");
  let activeLink;
  let activeTocLink;

  function keepVisible(link, container) {
    if (!link || !container?.clientHeight) return;
    const item = link.getBoundingClientRect();
    const viewport = container.getBoundingClientRect();
    if (item.top < viewport.top + 16 || item.bottom > viewport.bottom - 16) {
      container.scrollTop += item.top - viewport.top
        - (container.clientHeight - item.height) / 2;
    }
  }

  function update() {
    const tocLink = toc?.querySelector(".scroll-current > a");
    // API member headings may be deeper than the global tree. Use their parent.
    let item = tocLink?.closest("li");
    let link;
    while (item && !link) {
      link = sections.get(item.querySelector(":scope > a")?.hash);
      item = item.parentElement?.closest("li");
    }
    link ??= pageLink;
    if (link !== activeLink) {
      activeLink?.removeAttribute("aria-current");
      link?.setAttribute("aria-current", link === pageLink ? "page" : "location");
      activeLink = link;
      keepVisible(link, scroller);
    }
    if (tocLink !== activeTocLink) {
      activeTocLink?.removeAttribute("aria-current");
      tocLink?.setAttribute("aria-current", "location");
      activeTocLink = tocLink;
      keepVisible(tocLink, tocScroller);
    }
  }

  // Deactivation and activation happen together; update once after both events.
  let pending = false;
  function scheduleUpdate() {
    if (pending) return;
    pending = true;
    requestAnimationFrame(() => {
      pending = false;
      update();
    });
  }
  toc?.addEventListener("gumshoeActivate", scheduleUpdate);
  toc?.addEventListener("gumshoeDeactivate", scheduleUpdate);
  window.addEventListener("pageshow", scheduleUpdate);
  for (const [name, current, container] of [
    ["__navigation", () => activeLink, scroller],
    ["__toc", () => activeTocLink, tocScroller],
  ]) {
    document.querySelector(`input[name="${name}"]`)?.addEventListener("change", () => {
      requestAnimationFrame(() => keepVisible(current(), container));
    });
  }
  update();
});
