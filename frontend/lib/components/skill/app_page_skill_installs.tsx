// ========================================
// the skill page's install cards: the general NEAR contract skills, the
// skills that use the shit-1170 rex engine, and the REX agent itself.
//
// the cards are static: they show the commands for installing and listing
// each skill package, and for installing the agent. they do not fetch.
import { render } from "preact";
import { Copy, Check, ExternalLink } from "lucide-preact";
import { useState as use_state } from "preact/hooks";
import { SHIT_1170_MOUNT_ID } from "../../ts/shared/mount_ids";
// ========================================
// --- COMPONENT --------------------------
function SHIT_1170_COMPONENT_APP_PAGE_SKILL_INSTALLS_FUN() {
  const [copied, set_copied] = use_state("");

  const install_fun = async (skill_id: string, copy_message: string) => {
    try {
      await navigator.clipboard.writeText(copy_message);
      set_copied(skill_id);
      setTimeout(() => set_copied(""), 2000);
    } catch (err) {
      console.warn("[skill] the skill instructions could not be copied", err);
    }
  };

  const skill_cards = [
    {
      id: "near-kit-tool-box-skills",
      title: "NEAR KIT TOOL BOX SKILLS",
      description: "GENERAL SKILLS FOR INTERACTING WITH NEAR SMART CONTRACTS.",
      repository_url:
        "https://github.com/sleetplayground/near-kit-tool-box-skills",
      install_command:
        "bunx skills add sleetplayground/near-kit-tool-box-skills",
      list_command:
        "bunx skills add sleetplayground/near-kit-tool-box-skills --list",
      copy_message: [
        "Install the general skills for interacting with NEAR smart contracts:",
        "",
        "bunx skills add sleetplayground/near-kit-tool-box-skills",
        "",
        "To list the skills:",
        "",
        "bunx skills add sleetplayground/near-kit-tool-box-skills --list",
      ].join("\n"),
    },
    {
      id: "shit-1170-rex-engine-skills",
      title: "SHIT-1170 REX ENGINE SKILLS",
      description:
        "SPECIFIC SKILLS FOR INTERACTING WITH NEAR SMART CONTRACTS VIA THE SHIT-1170 REX ENGINE.",
      repository_url:
        "https://github.com/huggiesdotnear/shit-1170-rex-engine-skills",
      install_command:
        "bunx skills add huggiesdotnear/shit-1170-rex-engine-skills",
      list_command:
        "bunx skills add huggiesdotnear/shit-1170-rex-engine-skills --list",
      copy_message: [
        "Install the specific skills for interacting with NEAR smart contracts via the shit-1170 rex engine:",
        "",
        "bunx skills add huggiesdotnear/shit-1170-rex-engine-skills",
        "",
        "To list the skills:",
        "",
        "bunx skills add huggiesdotnear/shit-1170-rex-engine-skills --list",
      ].join("\n"),
    },
    {
      id: "rex-agent",
      title: "THE REX AGENT",
      description:
        "AN AGENT THAT READS THE CHAIN AND THE ENGINE, AND QUEUES TRANSACTIONS AND SWAPS FOR YOU TO SIGN.",
      repository_url:
        "https://github.com/huggiesdotnear/shit-1170-rex-engine-agent",
      repository_label: "AGENT",
      install_command:
        "bunx awesome-agents add huggiesdotnear/shit-1170-rex-engine-agent --agent rex",
      list_command:
        "bunx awesome-agents add huggiesdotnear/shit-1170-rex-engine-agent --list",
      copy_message: [
        "Install REX, the shit-1170 rex engine agent:",
        "",
        "bunx awesome-agents add huggiesdotnear/shit-1170-rex-engine-agent --agent rex",
        "",
        "To list the agent profiles in the repo:",
        "",
        "bunx awesome-agents add huggiesdotnear/shit-1170-rex-engine-agent --list",
      ].join("\n"),
    },
  ];

  return (
    <section>
      <h2>INSTALL SKILLS AND THE AGENT</h2>
      <p>
        ADD THESE SKILLS TO YOUR AGENT TO INTERACT WITH NEAR SMART CONTRACTS,
        OR INSTALL REX, THE AGENT THAT DOES IT FOR YOU.
      </p>
      <div class="skill-install-cards">
        {skill_cards.map((skill) => (
          <div key={skill.id} class="skill-install-card">
            <h3>{skill.title}</h3>
            <p>{skill.description}</p>
            <a
              class="skill-install-repository"
              href={skill.repository_url}
              target="_blank"
              rel="noreferrer"
            >
              VIEW {skill.repository_label ?? "SKILL"} REPOSITORY
              <ExternalLink size={14} />
            </a>
            <div class="skill-install-code-block">
              <code>
                <span class="skill-install-code-label"># add</span>
                {skill.install_command}
              </code>
              <code>
                <span class="skill-install-code-label"># --list</span>
                {skill.list_command}
              </code>
            </div>
            <button
              type="button"
              class="skill-install-copy"
              onClick={() => {
                void install_fun(skill.id, skill.copy_message);
              }}
              title="copy the install and list instructions"
            >
              {copied === skill.id ? <Check size={16} /> : <Copy size={16} />}
              {copied === skill.id ? "COPIED" : "COPY INSTRUCTIONS"}
            </button>
          </div>
        ))}
      </div>
    </section>
  );
}
// ========================================
// --- MOUNT ------------------------------
// self-mount guarded by id check.
const SELF_MOUNT_ID =
  SHIT_1170_MOUNT_ID.SHIT_1170_COMPONENT_APP_PAGE_SKILL_INSTALLS;
if (typeof document !== "undefined") {
  const el = document.getElementById(SELF_MOUNT_ID);
  if (el !== null)
    render(<SHIT_1170_COMPONENT_APP_PAGE_SKILL_INSTALLS_FUN />, el);
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
