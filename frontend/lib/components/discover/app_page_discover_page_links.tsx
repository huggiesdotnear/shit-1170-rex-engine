// ========================================
// the discover page's PAGES section: one link card per app page.
//
// --- WHY --- the nav only shows four pages, so a page that is not in
// the nav has no way in. this section is where every page gets a card,
// so a reader can reach the swap page, the unit conversion page and the
// skill page from the discover page.
//
// --- WHY --- each card carries the SAME lucide icon the nav uses for
// that page, so the page looks the same in the nav and on the card.
//
// --- WHY --- the list is a constant. a page list changes when the
// pages change, and a constant keeps the cards honest: there is one
// place to add a page.
// ========================================
import { render } from "preact";
import { SHIT_1170_MOUNT_ID } from "../../ts/shared/mount_ids";
import {
  ArrowDownUp,
  ArrowLeftRight,
  GraduationCap,
  KeyRound,
} from "lucide-preact";
import type { ComponentType } from "preact";
// ========================================
// ========================================
// --- CONFIG -----------------------------
interface SHIT_1170_PAGE_LINK_CARD {
  href: string;
  label: string;
  Icon: ComponentType<{ size?: number }>;
}
const SHIT_1170_PAGE_LINK_CARDS: ReadonlyArray<SHIT_1170_PAGE_LINK_CARD> = [
  { href: "swap.html", label: "SWAP", Icon: ArrowLeftRight },
  { href: "unit-conversion.html", label: "UNIT CONVERSION", Icon: ArrowDownUp },
  { href: "skill.html", label: "SKILL", Icon: GraduationCap },
  { href: "keys.html", label: "ACCESS KEYS", Icon: KeyRound },
];
// ========================================
// ========================================
// --- COMPONENT --------------------------
function SHIT_1170_COMPONENT_APP_PAGE_DISCOVER_PAGE_LINKS_FUN() {
  return (
    <div>
      {SHIT_1170_PAGE_LINK_CARDS.map((card) => (
        <a href={card.href}>
          <card.Icon size={20} />
          <span>{card.label}</span>
        </a>
      ))}
    </div>
  );
}
// ========================================
// --- MOUNT ------------------------------
// self-mount guarded by id check.
const SELF_MOUNT_ID =
  SHIT_1170_MOUNT_ID.SHIT_1170_COMPONENT_APP_PAGE_DISCOVER_PAGE_LINKS;
if (typeof document !== "undefined") {
  const el = document.getElementById(SELF_MOUNT_ID);
  if (el !== null)
    render(<SHIT_1170_COMPONENT_APP_PAGE_DISCOVER_PAGE_LINKS_FUN />, el);
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
