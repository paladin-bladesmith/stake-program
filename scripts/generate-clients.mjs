#!/usr/bin/env zx
import "zx/globals";
import * as k from "kinobi";
import { rootNodeFromAnchor } from "@kinobi-so/nodes-from-anchor";
import { renderVisitor as renderJavaScriptVisitor } from "@kinobi-so/renderers-js";
import { renderVisitor as renderRustVisitor } from "@kinobi-so/renderers-rust";
import { getAllProgramIdls } from "./utils.mjs";

// Instanciate Kinobi.
const [idl, ...additionalIdls] = getAllProgramIdls().map((idl) =>
  rootNodeFromAnchor(require(idl))
);
const kinobi = k.createFromRoot(idl, additionalIdls);

// Update programs.
kinobi.update(
  k.updateProgramsVisitor({
    stakeProgram: { name: "stake" },
  })
);

// Add PDA information.
kinobi.update(
  k.bottomUpTransformerVisitor([
    {
      select: "[programNode]stake",
      transform: (node) => {
        k.assertIsNode(node, "programNode");
        return {
          ...node,
          pdas: [
            k.pdaNode({
              name: "vault",
              seeds: [
                k.constantPdaSeedNodeFromString("utf8", "token-vault"),
                k.variablePdaSeedNode(
                  "authority",
                  k.publicKeyTypeNode(),
                  "Config authority"
                ),
              ],
            }),
          ],
        };
      },
    },
  ])
);

// Add missing types from the IDL.
kinobi.update(
  k.bottomUpTransformerVisitor([
    {
      // OptionalNonZeroPubkey -> NullableAddress
      select: (node) => {
        const names = ["authority", "slashAuthority"];
        return (
          names.includes(node.name) &&
          k.isNode(node, "structFieldTypeNode") &&
          k.isNode(node.type, "definedTypeLinkNode") &&
          node.type.name === "optionalNonZeroPubkey"
        );
      },
      transform: (node) => {
        k.assertIsNode(node, "structFieldTypeNode");
        return {
          ...node,
          type: k.definedTypeLinkNode("nullableAddress", "hooked"),
        };
      },
    },
    {
      // UnixTimestamp -> i64
      select: (node) => {
        const names = ["cooldownTimeSeconds", "deactivationTimestamp"];
        return (
          names.includes(node.name) &&
          k.isNode(node, "structFieldTypeNode") &&
          k.isNode(node.type, "definedTypeLinkNode") &&
          node.type.name === "unixTimestamp"
        );
      },
      transform: (node) => {
        k.assertIsNode(node, "structFieldTypeNode");
        return {
          ...node,
          type: k.numberTypeNode("i64"),
        };
      },
    },
  ])
);

// Update accounts.
kinobi.update(
  k.updateAccountsVisitor({
    config: {
      size: 136,
    },
  })
);

// Render JavaScript.
const jsClient = path.join(__dirname, "..", "clients", "js");
kinobi.accept(
  renderJavaScriptVisitor(path.join(jsClient, "src", "generated"), {
    prettier: require(path.join(jsClient, ".prettierrc.json")),
  })
);

// Render Rust.
const rustClient = path.join(__dirname, "..", "clients", "rust");
kinobi.accept(
  renderRustVisitor(path.join(rustClient, "src", "generated"), {
    formatCode: true,
    crateFolder: rustClient,
    toolchain: "+nightly-2023-10-05",
  })
);
