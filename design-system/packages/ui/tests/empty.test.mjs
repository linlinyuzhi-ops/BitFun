import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { Empty } from "../dist/index.js";

test("Empty exposes media, copy, and action anatomy", () => {
  const markup = renderToStaticMarkup(createElement(Empty, {
    actions: createElement("button", null, "Create"),
    description: "Created items will appear here.",
    icon: createElement("svg", { "aria-hidden": true }),
    title: "No items yet",
  }));

  assert.match(markup, /data-openbitfun-component="empty"/);
  assert.match(markup, /data-openbitfun-part="media"/);
  assert.match(markup, /data-openbitfun-part="title">No items yet/);
  assert.match(markup, /data-openbitfun-part="description">Created items will appear here\./);
  assert.match(markup, /data-openbitfun-part="actions"/);
});

test("Empty keeps decorative media and copy on the quiet content hierarchy", async () => {
  const styles = await readFile(
    new URL("../src/components/Empty/Empty.module.css", import.meta.url),
    "utf8",
  );
  const media = styles.match(/\.media\s*\{([^}]+)\}/)?.[1];
  const title = styles.match(/\.title\s*\{([^}]+)\}/)?.[1];

  assert.ok(media);
  assert.match(media, /color: var\(--openbitfun-component-empty-media\)/);
  assert.match(styles, /gap: var\(--openbitfun-layout-empty-gap\)/);
  assert.match(styles, /padding: var\(--openbitfun-layout-empty-padding-block\) var\(--openbitfun-layout-empty-padding-inline\)/);
  assert.match(media, /--_empty-media-size: var\(--openbitfun-layout-empty-media-size-md\)/);
  assert.match(media, /--_empty-icon-size: var\(--openbitfun-layout-empty-icon-size-md\)/);
  assert.match(styles, /\.media\[data-size="sm"\][^{]*\{[^}]*--_empty-icon-size: var\(--openbitfun-layout-empty-icon-size-sm\)/);
  assert.match(styles, /\.media\[data-size="lg"\][^{]*\{[^}]*--_empty-icon-size: var\(--openbitfun-layout-empty-icon-size-lg\)/);
  assert.match(styles, /inline-size: var\(--_empty-icon-size\)/);
  assert.match(styles, /block-size: var\(--_empty-icon-size\)/);
  assert.doesNotMatch(styles, /opacity:/);
  assert.ok(title);
  assert.match(title, /color: var\(--openbitfun-color-content-muted\)/);
  assert.match(title, /font-family: var\(--openbitfun-type-label-md-font-family\)/);
  assert.match(title, /font-size: var\(--openbitfun-type-label-md-font-size\)/);
  assert.match(title, /font-weight: var\(--openbitfun-type-label-md-font-weight\)/);
  assert.doesNotMatch(title, /color-content-primary|type-heading-card/);
});
