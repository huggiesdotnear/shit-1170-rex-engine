// ========================================
// the queue page's FastFS card: what a queued FastFS upload IS.
//
// a queued upload is not signed yet, so nothing is indexed and there is
// no file at any link. this card shows the file's FACTS — its name, its
// type, its size, and for a partial upload the chunk, the offset and
// the nonce — and the problems the indexer will drop it for.
//
// --- WHY THE CARD IS ITS OWN AND NOT THE TX PAGE'S: the tx page shows
// the SERVED file, with the link it is served at and the preview that
// link loads. this page shows what the batch asks to put up, and a
// queued upload has no served file yet. the two cards are not one card
// with a switch on, so this one lives in the queue page's folder.
//
// --- WHY THERE IS NO LINK AND NO NOTE ABOUT A LINK: the file is not
// indexed until the batch is signed, so a link here is a link to
// nothing, and a note about a link to nothing is noise on a page that
// is about what the batch will do. the queue page shows the file's
// facts and leaves the served file to the transaction page.
//
// --- WHY THERE IS NO PREVIEW: the preview is the file itself, drawn
// from the served link. there is no served link here, and what the
// queue page answers is what the file IS, not what it looks like.
//
// --- WHY THE LIMIT PROBLEMS ARE HERE: the indexer drops what breaks
// its rules SILENTLY, so a payload that breaks them describes a file
// that will never land. the problem has to be visible somewhere, and
// here is where the reader is looking.
//
// this component does NOT fetch. it renders what it is given.
import { fastfs_validate_fun } from "@sleet-js/fastnear-explorer-api-client";
import type { FASTFS_FILE_INFO } from "@sleet-js/fastnear-explorer-api-client";
// ========================================
// ========================================
// --- HELPERS ----------------------------
// the size of a file, in the units a reader thinks in.
//
// --- WHY THE UNITS ARE CHOSEN BY THE SIZE: a file of a few hundred
// bytes reads better in bytes, and a file of a few megabytes reads
// better in kilobytes or megabytes. one number in bytes would make the
// reader count zeroes.
export function size_text_fun(bytes: number): string {
  if (bytes < 1024) return `${bytes} BYTES`;
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}
// ========================================
// ========================================
// --- CARD -------------------------------
// the FastFS card: the file's facts and the problems it will be dropped
// for.
export function fastfs_card_fun(props: {
  info: FASTFS_FILE_INFO;
}): preact.JSX.Element {
  const info = props.info;
  const problems = fastfs_validate_fun(info);

  return (
    <div class="queue-fastfs-card">
      <div class="queue-fastfs-card-head">
        <span class="queue-fastfs-card-label">FASTFS UPLOAD</span>
        <span class="queue-fastfs-card-kind">
          {info.kind === "partial"
            ? "PARTIAL"
            : info.is_delete
              ? "DELETE"
              : "SIMPLE"}
        </span>
      </div>
      <div class="queue-fastfs-card-path">{info.relative_path}</div>
      <div class="queue-fastfs-card-facts">
        <span class="queue-fastfs-card-fact">
          <span class="queue-fastfs-card-fact-label">TYPE</span>
          <span class="queue-fastfs-card-fact-value">
            {info.mime_type === "" ? "—" : info.mime_type}
          </span>
        </span>
        {info.kind === "partial" ? (
          // --- WHY THE CHUNK AND THE FILE ARE TWO NUMBERS AND NOT ONE:
          // a partial upload carries a piece of a file, and the piece
          // and the whole are different sizes. a reader who sees one
          // number has to guess which it is.
          <>
            <span class="queue-fastfs-card-fact">
              <span class="queue-fastfs-card-fact-label">CHUNK</span>
              <span class="queue-fastfs-card-fact-value">
                {size_text_fun(info.size)}
              </span>
            </span>
            <span class="queue-fastfs-card-fact">
              <span class="queue-fastfs-card-fact-label">FILE</span>
              <span class="queue-fastfs-card-fact-value">
                {size_text_fun(info.total_size)}
              </span>
            </span>
            <span class="queue-fastfs-card-fact">
              <span class="queue-fastfs-card-fact-label">OFFSET</span>
              <span class="queue-fastfs-card-fact-value">{info.offset}</span>
            </span>
            <span class="queue-fastfs-card-fact">
              <span class="queue-fastfs-card-fact-label">NONCE</span>
              <span class="queue-fastfs-card-fact-value">{info.nonce}</span>
            </span>
          </>
        ) : (
          <span class="queue-fastfs-card-fact">
            <span class="queue-fastfs-card-fact-label">SIZE</span>
            <span class="queue-fastfs-card-fact-value">
              {size_text_fun(info.size)}
            </span>
          </span>
        )}
      </div>
      {problems.length > 0 && (
        <div class="queue-fastfs-card-problems">
          <span class="queue-fastfs-card-problems-label">
            THE INDEXER WILL DROP THIS — {problems.length} PROBLEM
            {problems.length === 1 ? "" : "S"}
          </span>
          <ul class="queue-fastfs-card-problems-list">
            {problems.map((problem) => (
              <li key={problem}>{problem}</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
