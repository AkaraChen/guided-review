/* <claim-card> component: a badge row, a title, optional muted claim text, and
   code evidence, divided from its siblings by a full-bleed hairline. Content is
   fully server-rendered light DOM; styles are document-level because slotted
   descendants (h3, evidence cards) could not be styled from a shadow tree. */
(() => {
  if (customElements.get("claim-card")) return;

  const style = document.createElement("style");
  style.textContent = `
    claim-card {
      display: block;
      min-width: 0;
      /* Divider bleeds through the section's 28px padding to the card edges */
      border-top: 1px solid var(--line-strong);
      margin: 0 -28px;
      padding: 18px 28px 0;
    }

    claim-card:first-child {
      border-top: 0;
      padding-top: 0;
    }

    claim-card .ref {
      display: inline-flex;
      flex-wrap: wrap;
      gap: 6px;
      margin-left: 10px;
      vertical-align: middle;
    }

    claim-card h3 .index {
      margin-right: 8px;
      color: var(--muted);
      font-family: var(--font-mono);
      font-size: 14px;
      letter-spacing: 0;
    }

    claim-card > p:not(.ref) {
      margin-top: 12px;
      color: var(--muted);
    }
  `;
  document.head.append(style);

  class ClaimCardElement extends HTMLElement {}

  customElements.define("claim-card", ClaimCardElement);
})();
