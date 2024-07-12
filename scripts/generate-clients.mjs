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

// Add PDA information.
kinobi.update(
  k.bottomUpTransformerVisitor([
    {
      select: "[programNode]paladinStakeProgram",
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
    {
      // Option<NonZeroU64> -> NullableU64
      select: "[structFieldTypeNode]deactivationTimestamp",
      transform: (node) => {
        k.assertIsNode(node, "structFieldTypeNode");
        return {
          ...node,
          type: k.definedTypeLinkNode("nullableU64", "hooked"),
        };
      },
    },
    {
      // [u8; 16] -> u128
      select: (node) => {
        const names = ["lastSeenHolderRewardsPerToken", "lastSeenStakeRewardsPerToken"];
        return (
          names.includes(node.name) &&
          k.isNode(node, "structFieldTypeNode") &&
          k.isNode(node.type, "arrayTypeNode")
        );
      },
      transform: (node) => {
        k.assertIsNode(node, "structFieldTypeNode");
        return {
          ...node,
          type: k.numberTypeNode("u128"),
        };
      },
    },
  ])
);

// Rename instruction arguments.
kinobi.update(
  k.bottomUpTransformerVisitor([
    {
      // DeactivateStake
      select: "[instructionNode]deactivateStake.[instructionArgumentNode]args",
      transform: (node) => {
        k.assertIsNode(node, "instructionArgumentNode");
        return {
          ...node,
          name: "amount",
        };
      },
    },
    {
      // Slash
      select: "[instructionNode]slash.[instructionArgumentNode]args",
      transform: (node) => {
        k.assertIsNode(node, "instructionArgumentNode");
        return {
          ...node,
          name: "amount",
        };
      },
    },
    {
      // StakeTokens
      select: "[instructionNode]stakeTokens.[instructionArgumentNode]args",
      transform: (node) => {
        k.assertIsNode(node, "instructionArgumentNode");
        return {
          ...node,
          name: "amount",
        };
      },
    },
    {
      // WithdrawInactiveStake
      select:
        "[instructionNode]withdrawInactiveStake.[instructionArgumentNode]args",
      transform: (node) => {
        k.assertIsNode(node, "instructionArgumentNode");
        return {
          ...node,
          name: "amount",
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
    stake: {
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
  })
);
