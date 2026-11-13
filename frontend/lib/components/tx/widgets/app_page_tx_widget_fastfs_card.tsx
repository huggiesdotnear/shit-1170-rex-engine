// ========================================
// the transaction page's FastFS card: what a FastFS upload that already
// happened IS.
//
// a FastFS upload is a function call to `__fastdata_fastfs` whose
// arguments are RAW BORSH BYTES. the client decodes those bytes into a
// file's facts, and this card shows them — the name, the type, the
// size — the link the file is served at, and a preview of it when the
// type allows one.
//
// --- WHY THE CARD IS GIVEN THE DECODED FILE AND NOT THE BYTES: the
// caller has to decode the bytes anyway — the transaction page's parser
// has already decoded them — so the decode happens ONCE, in the
// caller's hands, and the card is given the result. the card is about
// what a file IS, and how to show one, and not about what the bytes
// are.
//
// --- WHY THE DECODER IS THE CLIENT'S AND NOT A COPY HERE: the queue
// page shows a QUEUED upload, before it is signed, and the transaction
// page shows an ON-CHAIN one. the two must read the same bytes the
// same way, and two decoders drift. the fastnear client owns the borsh
// rules, so both pages ask it what the bytes mean.
//
// --- WHY THE CARD IS NOT SHARED WITH THE QUEUE PAGE: the queue page
// asks what the file IS — its facts, and nothing else — while this page
// shows the SERVED file, with the link it is served at and the preview
// that link loads. the two are not the same card with a switch on, and
// one card with a switch is two cards wearing one coat. so the queue
// page has its own card, in its own folder, and this one belongs to
// the page that shows the live file.
//
// --- WHY THE CARD IS ITS OWN COMPONENT AND NOT A BLOCK IN THE WIDGET:
// the rules about what the served file shows — what a preview needs,
// what the link means, what the indexer will drop — are a set of their
// own, and one copy of them is one copy to keep true.
//
// --- WHY THE PREVIEW ONLY WORKS ON-CHAIN: the file is served by the
// FastData indexer, and the indexer only knows about a file after the
// transaction is on chain. this page shows an ON-CHAIN upload, so the
// url has the file at it.
//
// --- WHY THE LIMIT PROBLEMS ARE HERE: the indexer drops what breaks
// its rules SILENTLY, so a payload that breaks them describes a file
// that will never land. the problem has to be visible somewhere, and
// here is where the reader is looking.
//
// --- WHY THE RECEIPT IS NOT ARGUED ABOUT HERE: a FastFS call fails on
// purpose — the receiver has no contract and does not need one, so the
// receipt comes back with `CodeDoesNotExist`. that fact is in the
// skill and in the docs. the page does not need to argue with the
// reader about it.
//
// this component does NOT fetch. it renders what it is given.
import {
  fastfs_can_display_fun,
  fastfs_file_uri_fun,
  fastfs_file_url_fun,
  fastfs_validate_fun,
} from "@sleet-js/fastnear-explorer-api-client";
import { shit_1170_code_block_fun } from "../code_block";
import type { FASTFS_FILE_INFO } from "@sleet-js/fastnear-explorer-api-client";
// ========================================
// ========================================
// --- CONSTANTS -------------------------
// how many characters of a text file's content are shown.
//
// --- WHY A LIMIT: a text file can be a megabyte, and a card with a
// megabyte of text in it is not a card — it is the page. the limit
// keeps the preview readable, and the size is shown right beside it, so
// the reader knows how much of the file there is.
const SHIT_1170_FASTFS_TEXT_PREVIEW_LIMIT = 2000;

// how many bytes of a text file are read for the preview.
//
// --- WHY A BYTE LIMIT AND NOT A CHARACTER LIMIT ON THE SOURCE: the
// limit is on the BYTES, because a byte is what the file is. a
// multi-byte character cut in half would put a broken glyph in the
// preview, so the bytes are cut on a character boundary afterwards.
const SHIT_1170_FASTFS_TEXT_PREVIEW_BYTES = 4000;
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

// the text of a text file, cut to a readable length.
//
// --- WHY THE CUT IS ON A CHARACTER BOUNDARY: the bytes are cut first,
// because that is the file. a multi-byte character cut in half would
// show as a broken glyph, so the cut bytes are decoded and the last
// character is dropped if it is a partial one.
function text_preview_fun(content: Uint8Array): {
  text: string;
  truncated: boolean;
} {
  const shown =
    content.length > SHIT_1170_FASTFS_TEXT_PREVIEW_BYTES
      ? content.slice(0, SHIT_1170_FASTFS_TEXT_PREVIEW_BYTES)
      : content;
  const decoder = new TextDecoder("utf-8", { fatal: false });
  let text = decoder.decode(shown);
  // --- WHY THE REPLACEMENT CHARACTER IS CHECKED FOR: a truncated
  // multi-byte character decodes to the replacement character, and
  // showing one is showing a broken character.
  if (text.endsWith("�")) text = text.slice(0, -1);
  const truncated =
    content.length > SHIT_1170_FASTFS_TEXT_PREVIEW_BYTES ||
    text.length > SHIT_1170_FASTFS_TEXT_PREVIEW_LIMIT;
  if (text.length > SHIT_1170_FASTFS_TEXT_PREVIEW_LIMIT) {
    text = text.slice(0, SHIT_1170_FASTFS_TEXT_PREVIEW_LIMIT);
  }
  return { text, truncated };
}
// ========================================
// ========================================
// --- PREVIEW ----------------------------
// the preview of the file, when its type allows one.
//
// --- WHY A QUEUED UPLOAD GETS NO IMAGE AND NO VIDEO: the file is
// served by the indexer, and the indexer only knows about a file once
// its transaction is on chain. a queued upload is not on chain, so the
// url has nothing at it. THIS PAGE SHOWS AN ON-CHAIN UPLOAD, so the
// url has the file at it and the preview draws.
function preview_fun(
  info: FASTFS_FILE_INFO,
  url: string,
): preact.JSX.Element | null {
  const kind = fastfs_can_display_fun(info.mime_type);
  if (kind === null) return null;

  if (kind === "text") {
    const preview = text_preview_fun(info.content);
    return (
      <div class="fastfs-preview is-text">
        {shit_1170_code_block_fun({ code: preview.text })}
        {preview.truncated && (
          <span class="fastfs-preview-truncated">
            TRUNCATED — THE FILE IS {size_text_fun(info.content.length)}
          </span>
        )}
      </div>
    );
  }

  // --- WHY --- AN EMPTY URL SHOWS NO PICTURE: the url is built from the
  // predecessor, and without one there is no file to point at. an `<img>`
  // with an empty source is a broken picture, and a broken picture says
  // less than a sentence.
  if (url === "") {
    return (
      <div class="fastfs-preview is-unavailable">
        <span class="fastfs-preview-note">
          NO {kind.toUpperCase()} PREVIEW — THE FILE IS SERVED UNDER THE ACCOUNT
          THAT SIGNED THE UPLOAD, AND THERE IS NONE
        </span>
      </div>
    );
  }

  if (kind === "image") {
    return (
      <div class="fastfs-preview is-image">
        <img class="fastfs-image" src={url} alt={info.relative_path} />
      </div>
    );
  }

  return (
    <div class="fastfs-preview is-video">
      {/* --- WHY THE CONTROLS ARE ON: a video the reader cannot pause
          or scrub is a thing that plays at them. */}
      <video class="fastfs-video" src={url} controls={true} />
    </div>
  );
}
// ========================================
// ========================================
// --- CARD -------------------------------
// the FastFS card: the file, the link, the preview, the notes.
export function fastfs_card_fun(props: {
  info: FASTFS_FILE_INFO;
  predecessor: string;
  receiver: string;
}): preact.JSX.Element {
  const info = props.info;
  // --- WHY --- THE LINK IS ONLY BUILT WHEN THERE IS A PREDECESSOR: the
  // predecessor is the HOST of the serving url, so without it the url
  // is `https://.fastfs.io/...` — a link to nothing, dressed up as a
  // link. a transaction with no signer has no account to serve the
  // file under, and the card says so.
  const has_predecessor = props.predecessor !== "";
  const url = has_predecessor
    ? fastfs_file_url_fun(props.predecessor, props.receiver, info.relative_path)
    : "";
  const uri = has_predecessor
    ? fastfs_file_uri_fun(props.predecessor, props.receiver, info.relative_path)
    : "";
  const problems = fastfs_validate_fun(info);

  return (
    <div class="fastfs-card">
      <div class="fastfs-head">
        <span class="fastfs-label">FASTFS UPLOAD</span>
        <span class="fastfs-kind">
          {info.kind === "partial"
            ? "PARTIAL"
            : info.is_delete
              ? "DELETE"
              : "SIMPLE"}
        </span>
      </div>
      <div class="fastfs-path">{info.relative_path}</div>
      <div class="fastfs-facts">
        <span class="fastfs-fact">
          <span class="fastfs-fact-label">TYPE</span>
          <span class="fastfs-fact-value">
            {info.mime_type === "" ? "—" : info.mime_type}
          </span>
        </span>
        {info.kind === "partial" ? (
          // --- WHY THE CHUNK AND THE FILE ARE TWO NUMBERS AND NOT ONE:
          // a partial upload carries a piece of a file, and the piece
          // and the whole are different sizes. a reader who sees one
          // number has to guess which it is.
          <>
            <span class="fastfs-fact">
              <span class="fastfs-fact-label">CHUNK</span>
              <span class="fastfs-fact-value">{size_text_fun(info.size)}</span>
            </span>
            <span class="fastfs-fact">
              <span class="fastfs-fact-label">FILE</span>
              <span class="fastfs-fact-value">
                {size_text_fun(info.total_size)}
              </span>
            </span>
            <span class="fastfs-fact">
              <span class="fastfs-fact-label">OFFSET</span>
              <span class="fastfs-fact-value">{info.offset}</span>
            </span>
            <span class="fastfs-fact">
              <span class="fastfs-fact-label">NONCE</span>
              <span class="fastfs-fact-value">{info.nonce}</span>
            </span>
          </>
        ) : (
          <span class="fastfs-fact">
            <span class="fastfs-fact-label">SIZE</span>
            <span class="fastfs-fact-value">{size_text_fun(info.size)}</span>
          </span>
        )}
      </div>
      {has_predecessor ? (
        <div class="fastfs-links">
          <a class="fastfs-url" href={url}>
            {url}
          </a>
          <span class="fastfs-uri">{uri}</span>
        </div>
      ) : (
        // --- WHY --- NO LINK WITHOUT A PREDECESSOR: the url is built
        // from the account that signed, and without one there is no
        // account to serve the file under. the transaction is on chain,
        // so the file is indexed somewhere, and this card does not know
        // where. it says so instead of showing a link to nothing.
        <div class="fastfs-links is-unknown">
          <span class="fastfs-note">
            NO LINK — THE FILE IS SERVED UNDER THE ACCOUNT THAT SIGNED THE
            UPLOAD, AND THIS TRANSACTION CARRIES NO SIGNER
          </span>
        </div>
      )}
      <div class="fastfs-note is-live">
        THE FILE IS SERVED FROM THE LINK ABOVE — THE INDEXER PICKED IT UP FROM
        THE TRANSACTION
      </div>
      {preview_fun(info, url)}
      {problems.length > 0 && (
        <div class="fastfs-problems">
          <span class="fastfs-problems-label">
            THE INDEXER WILL DROP THIS — {problems.length} PROBLEM
            {problems.length === 1 ? "" : "S"}
          </span>
          <ul class="fastfs-problems-list">
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
