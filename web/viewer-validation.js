/* SPDX-License-Identifier: MIT */
// Bounded JSON parsing and strict viewer-payload/replay validation.
// Depends only on viewer-core.js constants; performs no DOM work.
(function () {
  "use strict";

  var core = window.__ASCENSION_MAP_MODULES__.core;
  var MAX_JSON_BYTES = core.MAX_JSON_BYTES, MAX_TEXT_BYTES = core.MAX_TEXT_BYTES,
      MAX_NODES = core.MAX_NODES, MAX_EDGES = core.MAX_EDGES, MAX_CANDIDATES = core.MAX_CANDIDATES,
      MAX_REPLAY_FRAMES = core.MAX_REPLAY_FRAMES, MAX_DEPTH = core.MAX_DEPTH,
      STATUS_AVAILABILITY = core.STATUS_AVAILABILITY, STATUS_COMPLETENESS = core.STATUS_COMPLETENESS,
      STATUS_FRESHNESS = core.STATUS_FRESHNESS;

  function isRecord(value) {
    if (!value || typeof value !== "object" || Array.isArray(value)) return false;
    var proto = Object.getPrototypeOf(value);
    return proto === Object.prototype || proto === null;
  }

  function exactKeys(value, expected, path) {
    if (!isRecord(value)) throw new Error(path + " must be an object");
    var keys = Object.keys(value).sort();
    var wanted = expected.slice().sort();
    if (keys.length !== wanted.length || keys.some(function (key, index) { return key !== wanted[index]; })) {
      throw new Error(path + " has an unsupported shape");
    }
  }

  function textBytes(value, path) {
    if (typeof value !== "string" || value.length === 0) throw new Error(path + " must be non-empty text");
    var bytes = new TextEncoder().encode(value).byteLength;
    if (bytes > MAX_TEXT_BYTES || /[\u0000-\u001f\u007f\u202a-\u202e\u2066-\u2069\u200b-\u200f\u2060\ufeff]/u.test(value) || /\p{Cf}/u.test(value)) throw new Error(path + " contains a control or invisible formatting character");
    return value;
  }

  function canonicalText(value, path, allowEmpty) {
    if (allowEmpty && value === "") return value;
    var text = textBytes(value, path);
    if (!/^[\x20-\x7e]+$/u.test(text)) throw new Error(path + " must use printable ASCII canonical identity text");
    return text;
  }

  function bool(value, path) {
    if (typeof value !== "boolean") throw new Error(path + " must be boolean");
    return value;
  }

  function integer(value, path, min, max) {
    if (!Number.isInteger(value) || value < min || value > max) throw new Error(path + " is outside its bounded range");
    return value;
  }

  function statusText(value, path) {
    var status = textBytes(value, path);
    var parts = status.split(" / ");
    if (parts.length !== 3
        || !STATUS_AVAILABILITY.has(parts[0])
        || !STATUS_COMPLETENESS.has(parts[1])
        || !STATUS_FRESHNESS.has(parts[2])) {
      throw new Error(path + " contains an unsupported status");
    }
    return status;
  }

  function categoryText(value, path) {
    var category = textBytes(value, path);
    if (!/^[A-Za-z][A-Za-z0-9 _?/-]{0,63}$/u.test(category)) throw new Error(path + " has an unsupported category");
    return category;
  }

  function scanJsonString(input, index) {
    var start = index;
    if (input[index] !== '"') throw new Error("JSON object key is not a string");
    index += 1;
    while (index < input.length) {
      var code = input.charCodeAt(index);
      if (code === 34) {
        index += 1;
        return { value: JSON.parse(input.slice(start, index)), index: index };
      }
      if (code === 92) {
        index += 2;
        continue;
      }
      if (code < 32) throw new Error("JSON string contains a control character");
      index += 1;
    }
    throw new Error("JSON string is unterminated");
  }

  function skipSpace(input, index) {
    while (index < input.length && /\s/u.test(input[index])) index += 1;
    return index;
  }

  function rejectDuplicateKeys(input) {
    var cursor = 0;

    function scanValue(depth) {
      if (depth > MAX_DEPTH) throw new Error("JSON nesting exceeds the limit");
      cursor = skipSpace(input, cursor);
      var current = input[cursor];
      if (current === "{") {
        cursor += 1;
        cursor = skipSpace(input, cursor);
        var keys = new Set();
        if (input[cursor] === "}") { cursor += 1; return; }
        while (cursor < input.length) {
          cursor = skipSpace(input, cursor);
          var key = scanJsonString(input, cursor);
          cursor = skipSpace(input, key.index);
          if (input[cursor] !== ":") throw new Error("JSON object member is missing a colon");
          cursor += 1;
          if (keys.has(key.value)) throw new Error("JSON object contains a duplicate key");
          keys.add(key.value);
          scanValue(depth + 1);
          cursor = skipSpace(input, cursor);
          if (input[cursor] === "}") { cursor += 1; return; }
          if (input[cursor] !== ",") throw new Error("JSON object member is missing a comma");
          cursor += 1;
        }
        throw new Error("JSON object is unterminated");
      }
      if (current === "[") {
        cursor += 1;
        cursor = skipSpace(input, cursor);
        if (input[cursor] === "]") { cursor += 1; return; }
        while (cursor < input.length) {
          scanValue(depth + 1);
          cursor = skipSpace(input, cursor);
          if (input[cursor] === "]") { cursor += 1; return; }
          if (input[cursor] !== ",") throw new Error("JSON array member is missing a comma");
          cursor += 1;
        }
        throw new Error("JSON array is unterminated");
      }
      if (current === '"') {
        cursor = scanJsonString(input, cursor).index;
        return;
      }
      var start = cursor;
      while (cursor < input.length && !/[\s,\]}]/u.test(input[cursor])) cursor += 1;
      if (start === cursor) throw new Error("JSON value is missing");
    }

    scanValue(0);
    cursor = skipSpace(input, cursor);
    if (cursor !== input.length) throw new Error("JSON has trailing data");
  }

  function parseJsonBuffer(buffer, sourceLabel) {
    if (!buffer || buffer.byteLength > MAX_JSON_BYTES) throw new Error(sourceLabel + " is larger than 2 MiB");
    var input;
    try {
      input = new TextDecoder("utf-8", { fatal: true }).decode(buffer);
    } catch (error) {
      throw new Error(sourceLabel + " is not valid UTF-8");
    }
    rejectDuplicateKeys(input);
    try {
      return JSON.parse(input);
    } catch (error) {
      throw new Error(sourceLabel + " is not valid JSON");
    }
  }

  function deepFreeze(value) {
    if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
    Object.keys(value).forEach(function (key) { deepFreeze(value[key]); });
    return Object.freeze(value);
  }

  function validateCheckpoint(checkpoint) {
    exactKeys(checkpoint, ["reference", "run_id", "episode_id", "trajectory_id", "dispatchable"], "checkpoint");
    if (checkpoint.dispatchable !== false) throw new Error("checkpoint must be read only");
    ["run_id", "episode_id", "trajectory_id"].forEach(function (field) { textBytes(checkpoint[field], "checkpoint." + field); });
    var reference = checkpoint.reference;
    exactKeys(reference, ["schema", "reference_version", "handle", "occurrence", "boundary_kind", "boundary_phase", "assurance", "restore_verified"], "checkpoint reference");
    if (reference.schema !== "ascension.exact_checkpoint_reference.v1" || reference.reference_version !== "exact-checkpoint-reference-v1"
        || typeof reference.handle !== "string" || !/^ckpt-h1:[0-9a-f]{64}$/.test(reference.handle)
        || typeof reference.occurrence !== "string" || !/^[A-Za-z0-9_.:-]{1,256}$/.test(reference.occurrence)) throw new Error("unsupported checkpoint reference");
    ["boundary_kind", "boundary_phase"].forEach(function (field) {
      if (typeof reference[field] !== "string" || !reference[field].length || Array.from(reference[field]).length > 256) throw new Error("invalid checkpoint boundary");
    });
    if (!["public_observation_only", "capture_only", "restore_supported", "restore_verified", "continuation_certified"].includes(reference.assurance)
        || reference.restore_verified !== ["restore_verified", "continuation_certified"].includes(reference.assurance)) throw new Error("inconsistent checkpoint assurance");
  }

  function validatePayload(payload) {
    var fields = ["schema", "bundle_id", "snapshot_digest", "analysis_digest", "map", "aliases", "candidates", "historical", "complete", "available", "warnings"];
    if (Object.prototype.hasOwnProperty.call(payload, "checkpoint")) {
      fields.push("checkpoint");
      validateCheckpoint(payload.checkpoint);
    }
    exactKeys(payload, fields, "payload");
    if (payload.schema !== (fields.includes("checkpoint") ? "ascension-map-viewer-v2" : "ascension-map-viewer-v1")) throw new Error("payload.schema is unsupported");
    canonicalText(payload.bundle_id, "payload.bundle_id", true);
    if (!/^[a-f0-9]{64}$/u.test(payload.snapshot_digest)) throw new Error("payload.snapshot_digest must be lowercase SHA-256 text");
    if (!/^[a-f0-9]{64}$/u.test(payload.analysis_digest)) throw new Error("payload.analysis_digest must be lowercase SHA-256 text");
    bool(payload.historical, "payload.historical");
    bool(payload.complete, "payload.complete");
    bool(payload.available, "payload.available");
    if (!Array.isArray(payload.warnings) || payload.warnings.length > 128) throw new Error("payload.warnings has an unsupported shape");
    payload.warnings.forEach(function (warning, index) {
      if (typeof warning !== "string") throw new Error("payload.warnings[" + index + "] must be text");
      if (warning.length === 0) return;
      textBytes(warning, "payload.warnings[" + index + "]");
    });

    exactKeys(payload.map, ["identity", "status", "nodes", "edges"], "payload.map");
    textBytes(payload.map.identity, "payload.map.identity");
    statusText(payload.map.status, "payload.map.status");
    if (!Array.isArray(payload.map.nodes) || payload.map.nodes.length > MAX_NODES) throw new Error("payload.map.nodes exceeds the limit");
    if (!Array.isArray(payload.map.edges) || payload.map.edges.length > MAX_EDGES) throw new Error("payload.map.edges exceeds the limit");
    var nodeIds = new Set();
    payload.map.nodes.forEach(function (node, index) {
      exactKeys(node, ["id", "floor", "lane", "category", "current", "legal", "reachable", "visited"], "payload.map.nodes[" + index + "]");
      canonicalText(node.id, "payload.map.nodes[" + index + "].id", false);
      if (nodeIds.has(node.id)) throw new Error("payload.map.nodes contains a duplicate identity");
      nodeIds.add(node.id);
      integer(node.floor, "payload.map.nodes[" + index + "].floor", -32768, 32767);
      integer(node.lane, "payload.map.nodes[" + index + "].lane", -32768, 32767);
      categoryText(node.category, "payload.map.nodes[" + index + "].category");
      bool(node.current, "payload.map.nodes[" + index + "].current");
      bool(node.legal, "payload.map.nodes[" + index + "].legal");
      bool(node.reachable, "payload.map.nodes[" + index + "].reachable");
      bool(node.visited, "payload.map.nodes[" + index + "].visited");
    });
    var edgeKeys = new Set();
    payload.map.edges.forEach(function (edge, index) {
      exactKeys(edge, ["from", "to", "selected"], "payload.map.edges[" + index + "]");
      canonicalText(edge.from, "payload.map.edges[" + index + "].from", false);
      canonicalText(edge.to, "payload.map.edges[" + index + "].to", false);
      if (!nodeIds.has(edge.from) || !nodeIds.has(edge.to) || edge.from === edge.to) throw new Error("payload.map.edges has a malformed endpoint");
      var edgeKey = edge.from + "\u0000" + edge.to;
      if (edgeKeys.has(edgeKey)) throw new Error("payload.map.edges contains a duplicate direction");
      edgeKeys.add(edgeKey);
      bool(edge.selected, "payload.map.edges[" + index + "].selected");
    });

    exactKeys(payload.aliases, payload.map.nodes.map(function (node) { return node.id; }), "payload.aliases");
    var aliases = new Set();
    Object.keys(payload.aliases).forEach(function (id) {
      var alias = textBytes(payload.aliases[id], "payload.aliases[" + id + "]");
      if (aliases.has(alias)) throw new Error("payload.aliases contains a duplicate display alias");
      aliases.add(alias);
    });

    if (!Array.isArray(payload.candidates) || payload.candidates.length > MAX_CANDIDATES) throw new Error("payload.candidates has an unsupported shape");
    var candidateIds = new Set();
    payload.candidates.forEach(function (candidate, index) {
      exactKeys(candidate, ["id", "nodes", "summary", "selection"], "payload.candidates[" + index + "]");
      canonicalText(candidate.id, "payload.candidates[" + index + "].id", false);
      if (candidateIds.has(candidate.id)) throw new Error("payload.candidates contains a duplicate identity");
      candidateIds.add(candidate.id);
      if (!Array.isArray(candidate.nodes) || candidate.nodes.length === 0 || candidate.nodes.length > MAX_NODES) throw new Error("payload.candidates[" + index + "].nodes has an unsupported shape");
      var routeNodeIds = new Set();
      candidate.nodes.forEach(function (nodeId, nodeIndex) {
        canonicalText(nodeId, "payload.candidates[" + index + "].nodes[" + nodeIndex + "]", false);
        if (!nodeIds.has(nodeId) || routeNodeIds.has(nodeId)) throw new Error("payload.candidates references an invalid node sequence");
        routeNodeIds.add(nodeId);
      });
      for (var routeIndex = 1; routeIndex < candidate.nodes.length; routeIndex += 1) {
        if (!edgeKeys.has(candidate.nodes[routeIndex - 1] + "\u0000" + candidate.nodes[routeIndex])) throw new Error("payload.candidates contains a pair without a declared directed edge");
      }
      textBytes(candidate.summary, "payload.candidates[" + index + "].summary");
      textBytes(candidate.selection, "payload.candidates[" + index + "].selection");
    });
    return deepFreeze(payload);
  }

  function safeBundleId(value) {
    return typeof value === "string" && value.length > 0 && value.length <= MAX_TEXT_BYTES && !/[\\/\u0000-\u001f]/u.test(value) && !value.includes("..") && !value.includes("://");
  }

  function validateReplay(value) {
    var frames;
    if (Array.isArray(value)) {
      frames = value;
    } else {
      exactKeys(value, ["frames"], "replay response");
      frames = value.frames;
    }
    if (!Array.isArray(frames) || frames.length > MAX_REPLAY_FRAMES) throw new Error("replay response exceeds the frame limit");
    var seen = new Set();
    return frames.map(function (frame, index) {
      exactKeys(frame, ["bundle_id", "viewer"], "replay frame " + index);
      canonicalText(frame.bundle_id, "replay frame " + index + ".bundle_id", false);
      textBytes(frame.viewer, "replay frame " + index + ".viewer");
      if (!safeBundleId(frame.bundle_id) || seen.has(frame.bundle_id)) throw new Error("replay frame has an unsafe or duplicate bundle ID");
      if (frame.viewer.startsWith("/") || frame.viewer.includes("\\") || frame.viewer.includes("://") || frame.viewer.split("/").some(function (part) { return part === ".." || part === ""; })) throw new Error("replay frame has an unsafe viewer reference");
      seen.add(frame.bundle_id);
      return Object.freeze({ bundle_id: frame.bundle_id, viewer: frame.viewer });
    });
  }

  function validateAndFreezePayload(value, sourceLabel) {
    try { return validatePayload(value); } catch (error) { throw new Error(sourceLabel + ": " + error.message); }
  }

  window.__ASCENSION_MAP_MODULES__.validation = {
    isRecord: isRecord, exactKeys: exactKeys, textBytes: textBytes, canonicalText: canonicalText,
    bool: bool, integer: integer, statusText: statusText, categoryText: categoryText,
    parseJsonBuffer: parseJsonBuffer, deepFreeze: deepFreeze, validateCheckpoint: validateCheckpoint,
    validatePayload: validatePayload, safeBundleId: safeBundleId, validateReplay: validateReplay,
    validateAndFreezePayload: validateAndFreezePayload
  };
}());
