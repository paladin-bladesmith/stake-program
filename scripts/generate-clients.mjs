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

// Add missing types from the IDL.
kinobi.update(
  k.bottomUpTransformerVisitor([
    {
      select: "[programNode]stake",
      transform: (node) => {
        k.assertIsNode(node, "programNode");
        return {
          ...node,
          accounts: [
            ...node.accounts,
            // config
            k.accountNode({
              name: "config",
              size: 124,
              data: k.structTypeNode([
                k.structFieldTypeNode({
                  name: "accountType",
                  type: k.definedTypeLinkNode("AccountType"),
                }),
                k.structFieldTypeNode({
                  name: "authority",
                  type: k.publicKeyTypeNode(),
                }),
                k.structFieldTypeNode({
                  name: "slashAuthority",
                  type: k.publicKeyTypeNode(),
                }),
                k.structFieldTypeNode({
                  name: "vaultToken",
                  type: k.publicKeyTypeNode(),
                }),
                k.structFieldTypeNode({
                  name: "vaultBump",
                  type: k.numberTypeNode("u8"),
                }),
                k.structFieldTypeNode({
                  name: "cooldownTimeSeconds",
                  type: k.numberTypeNode("u64"),
                }),
                k.structFieldTypeNode({
                  name: "maxDeactivationBasisPoints",
                  type: k.numberTypeNode("u16"),
                }),
                k.structFieldTypeNode({
                  name: "tokenAmountDelegated",
                  type: k.numberTypeNode("u64"),
                }),

                k.structFieldTypeNode({
                  name: "totalStakeRewards",
                  type: k.numberTypeNode("u64"),
                }),
              ]),
            }),
          ],
          definedTypes: [
            ...node.definedTypes,
            // discriminator
            k.definedTypeNode({
              name: "accountType",
              type: k.enumTypeNode([
                k.enumEmptyVariantTypeNode("Uninitialized"),
                k.enumEmptyVariantTypeNode("Config"),
                k.enumEmptyVariantTypeNode("Stake"),
              ]),
            }),
          ],
          pdas: [
            k.pdaNode({
              name: "vault",
              seeds: [
                k.constantPdaSeedNodeFromString("utf8", "token-vault"),
                k.variablePdaSeedNode(
                  "configAuthority",
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

// Set account discriminators.
const key = (name) => ({
  field: "accountType",
  value: k.enumValueNode("AccountType", name),
});
kinobi.update(
  k.setAccountDiscriminatorFromFieldVisitor({
    config: key("config"),
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
