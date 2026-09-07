/* SPDX-License-Identifier: MIT */
(function () {
  "use strict";

  var SVG_NS = "http://www.w3.org/2000/svg";
  var VIEWBOX = { width: 1200, height: 780 };
  var COMPACT_VIEWBOX_WIDTH = 560;
  var MAX_JSON_BYTES = 2 * 1024 * 1024;
  var MAX_TEXT_BYTES = 512;
  var MAX_NODES = 1024;
  var MAX_EDGES = 8192;
  var MAX_CANDIDATES = 256;
  var MAX_REPLAY_FRAMES = 4096;
  var MAX_DEPTH = 64;
  var POLL_MS = 6000;
  var ZOOM_MIN = 0.55;
  var ZOOM_MAX = 4;
  var STATUS_WORDS = new Set([
    "AVAILABLE", "UNAVAILABLE", "COMPLETE", "INCOMPLETE", "CURRENT", "STALE",
    "HISTORICAL", "UNKNOWN", "DISCONNECTED"
  ]);
  var refs = {};
  var state = {
    payload: null,
    source: "none",
    historicalOverride: false,
    layout: null,
    selected: null,
    selectedCandidateId: null,
    search: "",
    zoom: 1,
    pan: { x: 0, y: 0 },
    overlays: { legal: true, reachable: true, visited: false, routes: true },
    live: { enabled: false, inFlight: false, timer: null, connected: false, failureCount: 0 },
    replay: { frames: [], index: -1, loadedIndex: -1, active: false },
    drag: { active: false, moved: false, x: 0, y: 0 }
  };

  function byId(id) { return document.getElementById(id); }

  function make(tag, attrs, value) {
    var node = document.createElement(tag);
    var attributes = attrs || {};
    Object.keys(attributes).forEach(function (key) {
      if (attributes[key] !== null && attributes[key] !== undefined) {
        node.setAttribute(key, String(attributes[key]));
      }
    });
    if (value !== undefined && value !== null) node.textContent = String(value);
    return node;
  }

  function makeSvg(tag, attrs, value) {
    var node = document.createElementNS(SVG_NS, tag);
    var attributes = attrs || {};
    Object.keys(attributes).forEach(function (key) {
      if (attributes[key] !== null && attributes[key] !== undefined) {
        node.setAttribute(key, String(attributes[key]));
      }
    });
    if (value !== undefined && value !== null) node.textContent = String(value);
    return node;
  }

  function removeChildren(node) {
    while (node.firstChild) node.removeChild(node.firstChild);
  }

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
    var words = status.toUpperCase().split(/[^A-Z]+/u).filter(Boolean);
    if (!words.length || words.some(function (word) { return !STATUS_WORDS.has(word); })) {
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

  function validatePayload(payload) {
    exactKeys(payload, ["schema", "bundle_id", "snapshot_digest", "analysis_digest", "map", "aliases", "candidates", "historical", "complete", "available", "warnings"], "payload");
    if (payload.schema !== "ascension-map-viewer-v1") throw new Error("payload.schema is unsupported");
    canonicalText(payload.bundle_id, "payload.bundle_id", true);
    if (!/^[a-f0-9]{64}$/u.test(payload.snapshot_digest)) throw new Error("payload.snapshot_digest must be lowercase SHA-256 text");
    if (!/^[a-f0-9]{64}$/u.test(payload.analysis_digest)) throw new Error("payload.analysis_digest must be lowercase SHA-256 text");
    bool(payload.historical, "payload.historical");
    bool(payload.complete, "payload.complete");
    bool(payload.available, "payload.available");
    if (!Array.isArray(payload.warnings) || payload.warnings.length > 128) throw new Error("payload.warnings has an unsupported shape");
    payload.warnings.forEach(function (warning, index) { textBytes(warning, "payload.warnings[" + index + "]"); });

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

  function bundledExample() {
    var nodes = [
      ["start", 0, 2, "start", false, true, true, false],
      ["route-left", 1, 0, "combat", false, true, true, false],
      ["route-center", 1, 2, "event", false, true, true, false],
      ["route-right", 1, 4, "shop", false, false, true, false],
      ["merge-early", 2, 1, "rest", false, true, true, false],
      ["merge-early-copy", 2, 1, "unknown", false, false, true, false],
      ["cross-high", 2, 3, "combat", false, false, true, false],
      ["side-disconnected", 2, 5, "treasure", false, false, false, false],
      ["left-elite", 3, 0, "elite", false, false, true, false],
      ["mid-event", 3, 2, "event", false, true, true, false],
      ["right-rest", 3, 4, "rest", false, false, true, false],
      ["old-visited", 3, 6, "combat", false, false, false, true],
      ["lane-left", 4, 1, "combat", false, false, true, false],
      ["lane-mid", 4, 3, "shop", false, true, true, false],
      ["lane-right", 4, 5, "unknown", false, false, false, false],
      ["quiet-merge", 5, 1, "rest", false, false, true, false],
      ["late-combat", 5, 3, "combat", true, true, true, false],
      ["far-branch", 5, 5, "event", false, false, false, false],
      ["boss-left", 6, 2, "elite", false, false, true, false],
      ["boss-right", 6, 4, "treasure", false, false, true, false],
      ["terminal", 7, 3, "boss", false, false, true, false]
    ].map(function (row) {
      return { id: row[0], floor: row[1], lane: row[2], category: row[3], current: row[4], legal: row[5], reachable: row[6], visited: row[7] };
    });
    var edges = [
      ["start", "route-left"], ["start", "route-center"], ["start", "route-right"],
      ["route-left", "merge-early"], ["route-left", "cross-high"], ["route-center", "merge-early"], ["route-center", "cross-high"], ["route-right", "cross-high"], ["route-right", "side-disconnected"],
      ["merge-early", "left-elite"], ["merge-early", "mid-event"], ["merge-early-copy", "mid-event"], ["cross-high", "mid-event"], ["cross-high", "right-rest"], ["side-disconnected", "old-visited"],
      ["left-elite", "lane-left"], ["left-elite", "lane-mid"], ["mid-event", "lane-left"], ["mid-event", "lane-mid"], ["right-rest", "lane-mid"], ["right-rest", "lane-right"],
      ["lane-left", "quiet-merge"], ["lane-left", "late-combat"], ["lane-mid", "quiet-merge"], ["lane-mid", "late-combat"], ["lane-right", "far-branch"],
      ["quiet-merge", "boss-left"], ["late-combat", "boss-left"], ["late-combat", "boss-right"], ["far-branch", "boss-right"], ["boss-left", "terminal"], ["boss-right", "terminal"]
    ].map(function (edge) { return { from: edge[0], to: edge[1], selected: edge[0] === "late-combat" && edge[1] === "boss-left" }; });
    var aliases = {};
    nodes.slice().sort(function (a, b) { return a.floor - b.floor || a.lane - b.lane || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0); }).forEach(function (node, index) { aliases[node.id] = "N" + String(index + 1).padStart(4, "0"); });
    return {
      schema: "ascension-map-viewer-v1",
      bundle_id: "synthetic-example-2026",
      snapshot_digest: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      analysis_digest: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      map: { identity: "synthetic-act-1-map", status: "AVAILABLE / COMPLETE / CURRENT", nodes: nodes, edges: edges },
      aliases: aliases,
      candidates: [
        { id: "route-amber", nodes: ["start", "route-center", "cross-high", "mid-event", "lane-mid", "late-combat", "boss-left", "terminal"], summary: "7 steps; keeps the current legal lane open", selection: "selected" },
        { id: "route-quiet", nodes: ["start", "route-left", "merge-early", "mid-event", "lane-left", "quiet-merge", "boss-left", "terminal"], summary: "7 steps; one visible rest before the terminal", selection: "heuristic" },
        { id: "route-wide", nodes: ["start", "route-right", "cross-high", "right-rest", "lane-mid", "late-combat", "boss-right", "terminal"], summary: "7 steps; includes a visible rest and treasure", selection: "comparison" }
      ],
      historical: false,
      complete: true,
      available: true,
      warnings: ["Synthetic example; no game or provider connection is attached."]
    };
  }

  function setNotice(message, kind) {
    removeChildren(refs.noticeRegion);
    if (!message) return;
    var notice = make("div", { class: "notice" + (kind === "error" ? " notice-error" : "") });
    notice.appendChild(make("strong", {}, kind === "error" ? "Artifact rejected" : "Viewer note"));
    notice.appendChild(make("span", {}, message));
    refs.noticeRegion.appendChild(notice);
  }

  function setConnectionBadge() {
    var badge = refs.connectionBadge;
    badge.className = "status-badge";
    var label = "Offline";
    if (state.historicalOverride || (state.payload && state.payload.historical)) {
      badge.classList.add("status-historical"); label = "Historical";
    } else if (state.source === "live" && state.live.connected) {
      badge.classList.add("status-live"); label = "Live";
    } else if (state.source === "live" && !state.live.connected) {
      badge.classList.add("status-disconnected"); label = "Disconnected";
    } else {
      badge.classList.add("status-offline");
    }
    badge.textContent = label;
  }

  function setStatusBadge() {
    var badge = refs.mapStatusBadge;
    badge.className = "status-badge status-neutral";
    if (!state.payload) {
      badge.textContent = "No map loaded";
      return;
    }
    var status = state.payload.map.status;
    var words = status.toUpperCase().split(/[^A-Z]+/u).filter(Boolean);
    if (words.includes("STALE") || words.includes("INCOMPLETE") || !state.payload.complete) badge.classList.add("status-warning");
    if (words.includes("CURRENT")) badge.classList.add("status-live");
    if (state.historicalOverride || state.payload.historical) badge.classList.add("status-historical");
    badge.textContent = state.historicalOverride ? "HISTORICAL · READ ONLY" : status;
  }

  function updateSourceLabels() {
    setConnectionBadge();
    setStatusBadge();
    if (!state.payload) {
      refs.bundleLabel.textContent = "Awaiting viewer.json";
      refs.scopeLabel.textContent = "No presentation loaded";
      refs.validationLabel.textContent = "Awaiting validation";
      return;
    }
    var source = state.source === "live" ? "LIVE" : state.source === "replay" ? "REPLAY" : state.source === "bundled" ? "EXAMPLE" : "OFFLINE";
    refs.bundleLabel.textContent = source + " · " + (state.payload.bundle_id || "unbound artifact");
    refs.scopeLabel.textContent = state.payload.map.identity;
    refs.validationLabel.textContent = "Validated viewer payload v1";
  }

  function categoryClass(category) {
    var normalized = category.toLowerCase().replace(/[^a-z0-9]+/g, "-");
    var known = ["start", "combat", "elite", "event", "mystery", "shop", "merchant", "rest", "treasure", "boss", "unknown"];
    return known.includes(normalized) ? normalized : "generic";
  }

  function categoryLabel(category) {
    return category.replace(/[-_]+/g, " ");
  }

  function stableNumber(value) {
    var hash = 0;
    for (var index = 0; index < value.length; index += 1) hash = ((hash << 5) - hash + value.charCodeAt(index)) | 0;
    return Math.abs(hash);
  }

  function viewWidth() {
    return state.layout && state.layout.viewWidth ? state.layout.viewWidth : VIEWBOX.width;
  }

  function updateMapViewBox() {
    var width = viewWidth();
    refs.mapSvg.setAttribute("viewBox", "0 0 " + width + " " + VIEWBOX.height);
    refs.mapBackground.setAttribute("width", String(width));
  }

  function createLayout(payload) {
    var floors = Array.from(new Set(payload.map.nodes.map(function (node) { return node.floor; }))).sort(function (a, b) { return a - b; });
    var lanes = Array.from(new Set(payload.map.nodes.map(function (node) { return node.lane; }))).sort(function (a, b) { return a - b; });
    var minFloor = floors.length ? floors[0] : 0;
    var maxFloor = floors.length ? floors[floors.length - 1] : 0;
    var minLane = lanes.length ? lanes[0] : 0;
    var maxLane = lanes.length ? lanes[lanes.length - 1] : 0;
    var compact = refs.mapStage && refs.mapStage.clientWidth < 560;
    var layoutWidth = compact ? COMPACT_VIEWBOX_WIDTH : VIEWBOX.width;
    var horizontalMargin = compact ? 36 : 100;
    var horizontalSpan = layoutWidth - horizontalMargin * 2;
    var floorRange = Math.max(1, maxFloor - minFloor);
    var laneRange = Math.max(1, maxLane - minLane);
    var positions = new Map();
    var groups = new Map();
    payload.map.nodes.forEach(function (node) {
      var key = node.floor + "\u0000" + node.lane;
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key).push(node);
    });
    groups.forEach(function (group) {
      group.sort(function (a, b) { return a.id < b.id ? -1 : a.id > b.id ? 1 : 0; });
      var offsetStep = Math.min(96, 176 / Math.max(1, group.length));
      group.forEach(function (node, index) {
        var x = horizontalMargin + ((node.lane - minLane) / laneRange) * horizontalSpan;
        var y = 690 - ((node.floor - minFloor) / floorRange) * 600;
        var offset = (index - (group.length - 1) / 2) * offsetStep;
        positions.set(node.id, { x: x + offset, y: y + Math.abs(offset) * 0.1 });
      });
    });
    return { positions: positions, floors: floors, viewWidth: layoutWidth, compact: compact, bounds: { minX: horizontalMargin - 28, minY: 52, maxX: layoutWidth - horizontalMargin + 28, maxY: 728 } };
  }

  function overviewPoint(position) {
    return { x: position.x * VIEWBOX.width / viewWidth(), y: position.y };
  }

  function edgeCurveGeometry(edge, index, positions) {
    var from = positions.get(edge.from);
    var to = positions.get(edge.to);
    var dx = to.x - from.x;
    var dy = to.y - from.y;
    var length = Math.max(1, Math.sqrt(dx * dx + dy * dy));
    var perpendicular = { x: -dy / length, y: dx / length };
    var offset = (stableNumber(edge.from + "->" + edge.to) % 5) - 2;
    var bend = offset * 10;
    var midX = (from.x + to.x) / 2 + perpendicular.x * bend;
    var midY = (from.y + to.y) / 2 + perpendicular.y * bend;
    return {
      path: "M " + from.x.toFixed(2) + " " + from.y.toFixed(2) + " Q " + midX.toFixed(2) + " " + midY.toFixed(2) + " " + to.x.toFixed(2) + " " + to.y.toFixed(2),
      midpoint: { x: (from.x + (2 * midX) + to.x) / 4, y: (from.y + (2 * midY) + to.y) / 4 }
    };
  }

  function routeNodeSet() {
    var set = new Set();
    if (!state.payload || !state.selectedCandidateId) return set;
    var candidate = state.payload.candidates.find(function (item) { return item.id === state.selectedCandidateId; });
    if (candidate) candidate.nodes.forEach(function (id) { set.add(id); });
    return set;
  }

  function routeEdgeSet() {
    var set = new Set();
    if (!state.payload || !state.selectedCandidateId) return set;
    var candidate = state.payload.candidates.find(function (item) { return item.id === state.selectedCandidateId; });
    if (!candidate) return set;
    for (var index = 1; index < candidate.nodes.length; index += 1) set.add(candidate.nodes[index - 1] + "\u0000" + candidate.nodes[index]);
    return set;
  }

  function isNodeDimmed(node) {
    if (state.overlays.legal && !node.legal && !node.current) return true;
    if (state.overlays.reachable && !node.reachable && !node.current) return true;
    if (state.overlays.visited && !node.visited && !node.current) return true;
    return false;
  }

  function isEdgeDimmed(edge) {
    if (!state.payload) return false;
    if (state.overlays.legal) {
      var from = state.payload.map.nodes.find(function (node) { return node.id === edge.from; });
      var to = state.payload.map.nodes.find(function (node) { return node.id === edge.to; });
      if (from && to && !to.reachable && !to.current) return true;
    }
    return false;
  }

  function createNodeShape(node, position) {
    var category = categoryClass(node.category);
    if (category === "elite" || category === "treasure") {
      return makeSvg("polygon", { class: "node-shape", points: [position.x + "," + (position.y - 18), (position.x + 18) + "," + position.y, position.x + "," + (position.y + 18), (position.x - 18) + "," + position.y].join(" ") });
    }
    if (category === "shop" || category === "merchant" || category === "rest") {
      return makeSvg("rect", { class: "node-shape", x: position.x - 16, y: position.y - 13, width: 32, height: 26, rx: 6 });
    }
    if (category === "event" || category === "mystery") {
      var points = [];
      for (var index = 0; index < 6; index += 1) {
        var angle = Math.PI / 6 + index * Math.PI / 3;
        points.push((position.x + Math.cos(angle) * 18).toFixed(2) + "," + (position.y + Math.sin(angle) * 18).toFixed(2));
      }
      return makeSvg("polygon", { class: "node-shape", points: points.join(" ") });
    }
    return makeSvg("circle", { class: "node-shape", cx: position.x, cy: position.y, r: category === "boss" ? 21 : 18 });
  }

  function renderGraph() {
    removeChildren(refs.edgeLayer);
    removeChildren(refs.nodeLayer);
    if (!state.payload || !state.layout) {
      refs.graphCount.textContent = "0 nodes · 0 edges";
      refs.emptyState.hidden = false;
      applyTransform();
      return;
    }
    refs.emptyState.hidden = true;
    refs.graphCount.textContent = state.payload.map.nodes.length + " nodes · " + state.payload.map.edges.length + " edges";
    var routeEdges = routeEdgeSet();
    state.payload.map.edges.forEach(function (edge, index) {
      var group = makeSvg("g", { class: "edge-group", tabindex: "0", role: "button", "aria-label": "Directed connection from " + edge.from + " to " + edge.to });
      group.dataset.edgeIndex = String(index);
      if (state.selected && state.selected.kind === "edge" && state.selected.index === index) group.classList.add("is-selected");
      if (state.overlays.routes && (edge.selected || routeEdges.has(edge.from + "\u0000" + edge.to))) group.classList.add("is-route");
      if (isEdgeDimmed(edge)) group.classList.add("is-dimmed");
      var geometry = edgeCurveGeometry(edge, index, state.layout.positions);
      var path = geometry.path;
      group.appendChild(makeSvg("path", { class: "edge-path", d: path }));
      group.appendChild(makeSvg("path", { class: "edge-hit", d: path, "aria-hidden": "true" }));
      group.appendChild(makeSvg("circle", { class: "edge-hit-point", cx: geometry.midpoint.x, cy: geometry.midpoint.y, r: 15, "aria-hidden": "true" }));
      var title = makeSvg("title", {}, "Connection " + edge.from + " to " + edge.to);
      group.insertBefore(title, group.firstChild);
      group.addEventListener("pointerdown", function (event) { event.stopPropagation(); });
      group.addEventListener("click", function (event) { event.stopPropagation(); selectEdge(index); });
      group.addEventListener("keydown", function (event) {
        if (event.key === "Enter" || event.key === " ") { event.preventDefault(); selectEdge(index); }
      });
      refs.edgeLayer.appendChild(group);
    });
    var routeNodes = routeNodeSet();
    var search = state.search.toLowerCase();
    state.payload.map.nodes.forEach(function (node) {
      var position = state.layout.positions.get(node.id);
      var alias = state.payload.aliases[node.id];
      var category = categoryClass(node.category);
      var group = makeSvg("g", { class: "node-group node-" + category, tabindex: "0", role: "button", "aria-label": alias + ", " + node.id + ", " + categoryLabel(node.category) });
      group.dataset.nodeId = node.id;
      if (state.selected && state.selected.kind === "node" && state.selected.id === node.id) group.classList.add("is-selected");
      if (node.current) group.classList.add("is-current");
      if (node.legal) group.classList.add("is-legal");
      if (node.reachable) group.classList.add("is-reachable");
      if (node.visited) group.classList.add("is-visited");
      if (category === "unknown" || node.category.toLowerCase() === "unknown") group.classList.add("is-unknown");
      if (state.payload.map.status.toUpperCase().includes("STALE") || !state.payload.complete) group.classList.add("is-stale");
      if (state.overlays.routes && routeNodes.has(node.id)) group.classList.add("is-candidate");
      if (search && (node.id.toLowerCase().includes(search) || alias.toLowerCase().includes(search))) group.classList.add("is-search-match");
      if (isNodeDimmed(node)) group.classList.add("is-dimmed");
      group.appendChild(makeSvg("circle", { class: "node-hit", cx: position.x, cy: position.y, r: 32, "aria-hidden": "true" }));
      group.appendChild(createNodeShape(node, position));
      if (category === "unknown" || node.category.toLowerCase() === "unknown") group.appendChild(makeSvg("text", { class: "node-unknown-mark", x: position.x, y: position.y + 1 }, "?"));
      group.appendChild(makeSvg("text", { class: "node-label", x: position.x, y: position.y + 39 }, alias));
      group.appendChild(makeSvg("text", { class: "node-category-label", x: position.x, y: position.y + 53 }, categoryLabel(node.category)));
      var title = makeSvg("title", {}, alias + " · " + node.id);
      group.insertBefore(title, group.firstChild);
      group.addEventListener("pointerdown", function (event) { event.stopPropagation(); });
      group.addEventListener("click", function (event) { event.stopPropagation(); if (!state.drag.moved) selectNode(node.id); });
      group.addEventListener("keydown", function (event) { handleNodeKey(event, node.id); });
      refs.nodeLayer.appendChild(group);
    });
    applyTransform();
  }

  function renderOverview() {
    removeChildren(refs.overviewContent);
    if (!state.payload || !state.layout) {
      refs.overviewViewport.setAttribute("x", "0"); refs.overviewViewport.setAttribute("y", "0"); refs.overviewViewport.setAttribute("width", String(VIEWBOX.width)); refs.overviewViewport.setAttribute("height", String(VIEWBOX.height));
      refs.overviewScale.textContent = "Fit";
      return;
    }
    var positions = state.layout.positions;
    state.payload.map.edges.forEach(function (edge) {
      var from = overviewPoint(positions.get(edge.from)); var to = overviewPoint(positions.get(edge.to));
      refs.overviewContent.appendChild(makeSvg("line", { class: "overview-line", x1: from.x, y1: from.y, x2: to.x, y2: to.y }));
    });
    state.payload.map.nodes.forEach(function (node) {
      var position = overviewPoint(positions.get(node.id));
      var circle = makeSvg("circle", { class: "overview-node " + (node.current ? "overview-current" : node.legal ? "overview-legal" : "overview-normal"), cx: position.x, cy: position.y, r: node.current ? 8 : 5 });
      refs.overviewContent.appendChild(circle);
    });
    updateOverviewViewport();
  }

  function updateOverviewViewport() {
    var mapWidth = viewWidth();
    var width = mapWidth / state.zoom;
    var height = VIEWBOX.height / state.zoom;
    var x = Math.max(0, Math.min(mapWidth - width, -state.pan.x / state.zoom));
    var overviewScaleX = VIEWBOX.width / mapWidth;
    refs.overviewViewport.setAttribute("x", x * overviewScaleX);
    refs.overviewViewport.setAttribute("y", Math.max(0, Math.min(VIEWBOX.height - height, -state.pan.y / state.zoom)));
    refs.overviewViewport.setAttribute("width", Math.min(mapWidth, width) * overviewScaleX);
    refs.overviewViewport.setAttribute("height", Math.min(VIEWBOX.height, height));
    refs.overviewScale.textContent = Math.round(state.zoom * 100) + "%";
  }

  function applyTransform() {
    refs.mapContent.setAttribute("transform", "translate(" + state.pan.x.toFixed(2) + " " + state.pan.y.toFixed(2) + ") scale(" + state.zoom.toFixed(3) + ")");
    refs.zoomReadout.textContent = Math.round(state.zoom * 100) + "%";
    updateOverviewViewport();
  }

  function fitMap() { state.zoom = 1; state.pan = { x: 0, y: 0 }; applyTransform(); }

  function zoomAt(nextZoom, clientX, clientY) {
    var rect = refs.mapSvg.getBoundingClientRect();
    var x = ((clientX - rect.left) / Math.max(1, rect.width)) * viewWidth();
    var y = ((clientY - rect.top) / Math.max(1, rect.height)) * VIEWBOX.height;
    var oldZoom = state.zoom;
    state.zoom = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, nextZoom));
    state.pan.x = x - (state.zoom / oldZoom) * (x - state.pan.x);
    state.pan.y = y - (state.zoom / oldZoom) * (y - state.pan.y);
    applyTransform();
  }

  function zoomStep(direction) {
    var rect = refs.mapSvg.getBoundingClientRect();
    zoomAt(state.zoom * (direction > 0 ? 1.22 : 0.82), rect.left + rect.width / 2, rect.top + rect.height / 2);
  }

  function findNode(id) {
    return state.payload ? state.payload.map.nodes.find(function (node) { return node.id === id; }) : null;
  }

  function findNodeElement(id) {
    return Array.from(refs.nodeLayer.children).find(function (node) { return node.dataset.nodeId === id; }) || null;
  }

  function appendFact(list, label, value) {
    var wrapper = make("div");
    wrapper.appendChild(make("dt", {}, label));
    wrapper.appendChild(make("dd", {}, value));
    list.appendChild(wrapper);
  }

  function stateChip(label, value, extraClass) {
    return make("span", { class: "state-chip " + (value ? "is-true" : "is-false") + (extraClass ? " " + extraClass : "") }, label + ": " + (value ? "yes" : "no"));
  }

  function connectionRows(nodeId) {
    var rows = [];
    state.payload.map.edges.forEach(function (edge) {
      if (edge.from === nodeId) rows.push({ direction: "→", value: state.payload.aliases[edge.to], type: "out" });
      if (edge.to === nodeId) rows.push({ direction: "←", value: state.payload.aliases[edge.from], type: "in" });
    });
    return rows;
  }

  function appendCandidateList(parent) {
    var heading = make("p", { class: "inspector-subhead" }, "Candidate comparisons");
    parent.appendChild(heading);
    if (!state.payload.candidates.length) {
      parent.appendChild(make("p", { class: "muted-copy" }, "No bounded route candidates were recorded for this frame."));
      return;
    }
    var list = make("div", { class: "candidate-list" });
    state.payload.candidates.forEach(function (candidate) {
      var button = make("button", { type: "button", class: "candidate-button" + (state.selectedCandidateId === candidate.id ? " is-selected" : ""), "aria-pressed": state.selectedCandidateId === candidate.id ? "true" : "false" });
      button.dataset.candidateId = candidate.id;
      var top = make("span", { class: "candidate-top" });
      top.appendChild(make("strong", {}, candidate.id));
      top.appendChild(make("small", {}, candidate.selection));
      button.appendChild(top);
      button.appendChild(make("span", { class: "candidate-summary" }, candidate.summary));
      button.addEventListener("click", function () {
        state.selectedCandidateId = state.selectedCandidateId === candidate.id ? null : candidate.id;
        renderAll();
      });
      list.appendChild(button);
    });
    parent.appendChild(list);
  }

  function renderInspector() {
    removeChildren(refs.inspectorBody);
    if (!state.payload || !state.selected) {
      refs.inspectorHeading.textContent = "Nothing selected";
      if (!state.payload) {
        refs.inspectorBody.appendChild(make("p", { class: "muted-copy" }, "Select a node or directional connection to see its recorded facts. Viewer selections never dispatch game actions."));
      } else {
        refs.inspectorBody.appendChild(make("p", { class: "inspector-lead" }, state.payload.map.nodes.length + " visible nodes and " + state.payload.map.edges.length + " directed connections are present in this frame."));
        refs.inspectorBody.appendChild(make("p", { class: "muted-copy" }, "Select any graph element to inspect it. Route choices are annotations only; this viewer has no game action endpoints."));
        appendCandidateList(refs.inspectorBody);
      }
      return;
    }
    if (state.selected.kind === "node") {
      var node = findNode(state.selected.id);
      if (!node) { state.selected = null; renderInspector(); return; }
      var alias = state.payload.aliases[node.id];
      refs.inspectorHeading.textContent = "Node inspection";
      refs.inspectorBody.appendChild(make("p", { class: "inspector-id" }, node.id));
      refs.inspectorBody.appendChild(make("p", { class: "inspector-alias" }, alias + " · " + categoryLabel(node.category)));
      var facts = make("dl", { class: "fact-list" });
      appendFact(facts, "Floor", node.floor); appendFact(facts, "Lane", node.lane); appendFact(facts, "Category", node.category); appendFact(facts, "Canonical", node.id);
      refs.inspectorBody.appendChild(facts);
      var states = make("div", { class: "state-list", "aria-label": "Recorded node states" });
      states.appendChild(stateChip("current", node.current, "is-current")); states.appendChild(stateChip("legal", node.legal)); states.appendChild(stateChip("reachable", node.reachable)); states.appendChild(stateChip("visited", node.visited));
      refs.inspectorBody.appendChild(states);
      var connections = connectionRows(node.id);
      refs.inspectorBody.appendChild(make("p", { class: "inspector-subhead" }, "Connections · " + connections.length));
      if (!connections.length) refs.inspectorBody.appendChild(make("p", { class: "muted-copy" }, "No recorded connections touch this node."));
      else {
        var list = make("ul", { class: "connection-list" });
        connections.forEach(function (row) { var item = make("li"); item.appendChild(make("span", { class: "direction" }, row.direction)); item.appendChild(make("span", {}, row.value)); item.appendChild(make("span", { class: "connection-type" }, row.type)); list.appendChild(item); });
        refs.inspectorBody.appendChild(list);
      }
      var memberships = state.payload.candidates.filter(function (candidate) { return candidate.nodes.includes(node.id); });
      if (memberships.length) {
        refs.inspectorBody.appendChild(make("p", { class: "inspector-subhead" }, "Candidate membership"));
        refs.inspectorBody.appendChild(make("p", { class: "muted-copy" }, memberships.map(function (candidate) { return candidate.id; }).join(" · ")));
      }
    } else {
      var edge = state.payload.map.edges[state.selected.index];
      if (!edge) { state.selected = null; renderInspector(); return; }
      refs.inspectorHeading.textContent = "Edge inspection";
      refs.inspectorBody.appendChild(make("p", { class: "inspector-lead" }, "Directed connection"));
      var edgeFacts = make("dl", { class: "fact-list" });
      appendFact(edgeFacts, "From", state.payload.aliases[edge.from]); appendFact(edgeFacts, "To", state.payload.aliases[edge.to]); appendFact(edgeFacts, "From ID", edge.from); appendFact(edgeFacts, "To ID", edge.to);
      refs.inspectorBody.appendChild(edgeFacts);
      refs.inspectorBody.appendChild(make("div", { class: "state-list" }, ""));
      var edgeState = make("div", { class: "state-list" }); edgeState.appendChild(stateChip("recorded route", edge.selected)); refs.inspectorBody.appendChild(edgeState);
      refs.inspectorBody.appendChild(make("p", { class: "muted-copy historical-note" }, "Inspect only. A selected edge does not authorize a game action."));
    }
    if (state.historicalOverride || state.payload.historical) refs.inspectorBody.appendChild(make("p", { class: "historical-note" }, "Historical frame · recorded authorization only. No current action is available."));
  }

  function renderSearchResults() {
    removeChildren(refs.searchResults);
    var query = state.search.trim().toLowerCase();
    if (!query || !state.payload) { refs.searchResults.hidden = true; return; }
    var matches = state.payload.map.nodes.filter(function (node) { return node.id.toLowerCase().includes(query) || state.payload.aliases[node.id].toLowerCase().includes(query); }).slice(0, 8);
    if (!matches.length) {
      refs.searchResults.appendChild(make("div", { class: "search-result" }, "No matching canonical ID or alias"));
    } else {
      matches.forEach(function (node) {
        var result = make("button", { type: "button", class: "search-result", role: "option" });
        result.appendChild(make("strong", {}, state.payload.aliases[node.id]));
        result.appendChild(make("small", {}, node.id + " · floor " + node.floor + ", lane " + node.lane));
        result.addEventListener("click", function () { refs.search.value = state.payload.aliases[node.id]; state.search = refs.search.value; refs.searchResults.hidden = true; selectNode(node.id, true); });
        refs.searchResults.appendChild(result);
      });
    }
    refs.searchResults.hidden = false;
  }

  function renderReplayControls() {
    removeChildren(refs.replaySelect);
    if (!state.replay.frames.length) {
      refs.replaySelect.appendChild(make("option", { value: "" }, "No replay frames"));
      refs.replaySelect.disabled = true;
      refs.replayCount.textContent = "No frames";
      refs.replayNote.textContent = state.source === "live" ? "No replay index is available on this live source." : "Historical frames retain the knowledge and authorization available at that decision.";
      return;
    }
    state.replay.frames.forEach(function (frame, index) {
      refs.replaySelect.appendChild(make("option", { value: String(index) }, "Frame " + String(index + 1).padStart(2, "0") + " · " + frame.bundle_id));
    });
    refs.replaySelect.disabled = false;
    refs.replaySelect.value = state.replay.index >= 0 ? String(state.replay.index) : "";
    refs.replayCount.textContent = state.replay.frames.length + " frame" + (state.replay.frames.length === 1 ? "" : "s");
  }

  function renderAll() {
    updateSourceLabels();
    renderGraph();
    renderOverview();
    renderInspector();
    renderSearchResults();
    renderReplayControls();
    Object.keys(state.overlays).forEach(function (name) {
      var toggle = document.querySelector('[data-overlay="' + name + '"]');
      if (!toggle) return;
      toggle.classList.toggle("is-on", state.overlays[name]);
      toggle.setAttribute("aria-pressed", state.overlays[name] ? "true" : "false");
    });
  }

  function selectNode(id, focus) {
    if (!findNode(id)) return;
    state.selected = { kind: "node", id: id };
    renderAll();
    if (focus) {
      var nodeElement = findNodeElement(id);
      if (nodeElement) nodeElement.focus();
    }
  }

  function selectEdge(index) {
    if (!state.payload || !state.payload.map.edges[index]) return;
    state.selected = { kind: "edge", index: index };
    renderAll();
    var edgeElement = Array.from(refs.edgeLayer.children).find(function (node) { return Number(node.dataset.edgeIndex) === index; });
    if (edgeElement) edgeElement.focus();
  }

  function connectedNodes(id) {
    if (!state.payload) return [];
    var ids = [];
    state.payload.map.edges.forEach(function (edge) { if (edge.from === id) ids.push(edge.to); if (edge.to === id) ids.push(edge.from); });
    return Array.from(new Set(ids)).sort();
  }

  function handleNodeKey(event, id) {
    if (event.key === "Enter" || event.key === " ") { event.preventDefault(); selectNode(id); return; }
    if (!["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(event.key)) return;
    var neighbors = connectedNodes(id);
    if (!neighbors.length) return;
    event.preventDefault();
    var selectedIndex = neighbors.indexOf(state.selected && state.selected.id);
    var direction = event.key === "ArrowUp" || event.key === "ArrowLeft" ? -1 : 1;
    var nextIndex = selectedIndex < 0 ? (direction > 0 ? 0 : neighbors.length - 1) : (selectedIndex + direction + neighbors.length) % neighbors.length;
    selectNode(neighbors[nextIndex], true);
  }

  function applyPayload(payload, source, historicalOverride) {
    var identityChanged = !state.payload || state.payload.map.identity !== payload.map.identity || state.payload.bundle_id !== payload.bundle_id;
    state.payload = payload;
    state.source = source;
    state.historicalOverride = Boolean(historicalOverride);
    state.layout = createLayout(payload);
    updateMapViewBox();
    if (identityChanged) {
      state.selected = null;
      state.zoom = 1;
      state.pan = { x: 0, y: 0 };
    } else if (state.selected && state.selected.kind === "node" && !findNode(state.selected.id)) {
      state.selected = null;
    }
    var selectedCandidate = payload.candidates.find(function (candidate) { return candidate.selection.toLowerCase() === "selected"; });
    state.selectedCandidateId = selectedCandidate ? selectedCandidate.id : null;
    refs.emptyState.hidden = true;
    setNotice(payload.warnings.length ? payload.warnings[0] : "", "info");
    renderAll();
  }

  async function readFile(file) {
    try {
      var payload = validateAndFreezePayload(parseJsonBuffer(await file.arrayBuffer(), file.name || "viewer.json"), file.name || "viewer.json");
      applyPayload(payload, "offline", false);
    } catch (error) {
      setNotice(error.message, "error");
      refs.validationLabel.textContent = "Artifact rejected";
    }
  }

  async function pollCurrent() {
    if (!state.live.enabled || state.live.inFlight || state.replay.active) return;
    state.live.inFlight = true;
    try {
      var response = await fetch("/api/current", { cache: "no-store", headers: { Accept: "application/json" } });
      if (!response.ok) throw new Error("live source returned HTTP " + response.status);
      var payload = validateAndFreezePayload(parseJsonBuffer(await response.arrayBuffer(), "live /api/current"), "live /api/current");
      if (!payload.bundle_id) throw new Error("live /api/current must provide a bound bundle ID");
      applyPayload(payload, "live", false);
      state.live.connected = true;
      state.live.failureCount = 0;
      setNotice(payload.warnings.length ? payload.warnings[0] : "", "info");
      setConnectionBadge();
    } catch (error) {
      state.live.connected = false;
      state.live.failureCount += 1;
      setConnectionBadge();
      if (state.payload) setNotice("Live source disconnected; the last complete frame remains visible until a fresh valid frame arrives.", "error");
    } finally {
      state.live.inFlight = false;
    }
  }

  async function loadReplayIndex() {
    if (!state.live.enabled) return;
    try {
      var response = await fetch("/api/replay", { cache: "no-store", headers: { Accept: "application/json" } });
      if (!response.ok) throw new Error("replay source returned HTTP " + response.status);
      var frames = validateReplay(parseJsonBuffer(await response.arrayBuffer(), "live /api/replay"));
      state.replay.frames = frames;
      state.replay.index = -1;
      renderReplayControls();
    } catch (error) {
      state.replay.frames = [];
      state.replay.index = -1;
      renderReplayControls();
      refs.replayNote.textContent = "Replay index unavailable; current live viewing remains independent.";
    }
  }

  async function loadReplayFrame(index) {
    if (!state.replay.frames[index] || state.replay.loadedIndex === index) return;
    var frame = state.replay.frames[index];
    try {
      var response = await fetch("/api/frame/" + encodeURIComponent(frame.bundle_id), { cache: "no-store", headers: { Accept: "application/json" } });
      if (!response.ok) throw new Error("replay frame returned HTTP " + response.status);
      var payload = validateAndFreezePayload(parseJsonBuffer(await response.arrayBuffer(), "replay frame"), "replay frame");
      if (!payload.bundle_id || payload.bundle_id !== frame.bundle_id) throw new Error("replay frame bundle ID does not match its descriptor");
      state.replay.index = index;
      state.replay.loadedIndex = index;
      state.replay.active = true;
      applyPayload(payload, "replay", true);
      refs.replayNote.textContent = "Historical frame " + (index + 1) + " · recorded authorization only.";
      renderReplayControls();
    } catch (error) {
      setNotice(error.message, "error");
      refs.replayNote.textContent = "Could not load this historical frame; the current map remains intact.";
      refs.replaySelect.value = state.replay.index >= 0 ? String(state.replay.index) : "";
    }
  }

  function leaveReplay() {
    if (!state.replay.active) return;
    state.replay.active = false;
    state.replay.loadedIndex = -1;
    state.replay.index = -1;
    if (state.live.enabled) pollCurrent();
  }

  function initReferences() {
    refs.connectionBadge = byId("connection-badge"); refs.mapStatusBadge = byId("map-status-badge"); refs.bundleLabel = byId("bundle-label"); refs.scopeLabel = byId("scope-label"); refs.validationLabel = byId("validation-label");
    refs.noticeRegion = byId("notice-region"); refs.mapStage = byId("map-stage"); refs.mapSvg = byId("map-svg"); refs.mapBackground = refs.mapSvg.querySelector(".map-background"); refs.mapContent = byId("map-content"); refs.edgeLayer = byId("edge-layer"); refs.nodeLayer = byId("node-layer"); refs.emptyState = byId("empty-state"); refs.graphCount = byId("graph-count"); refs.zoomReadout = byId("zoom-readout");
    refs.overviewContent = byId("overview-content"); refs.overviewViewport = byId("overview-viewport"); refs.overviewScale = byId("overview-scale"); refs.overviewNote = byId("overview-note"); refs.inspectorHeading = byId("inspector-heading"); refs.inspectorBody = byId("inspector-body"); refs.search = byId("node-search"); refs.searchResults = byId("search-results"); refs.artifactFile = byId("artifact-file"); refs.replaySelect = byId("replay-select"); refs.replayCount = byId("replay-count"); refs.replayNote = byId("replay-note"); refs.keyboardDialog = byId("keyboard-dialog");
  }

  function initEvents() {
    document.querySelectorAll("[data-action]").forEach(function (button) {
      button.addEventListener("click", function () {
        var action = button.getAttribute("data-action");
        if (action === "fit") fitMap();
        if (action === "zoom-in") zoomStep(1);
        if (action === "zoom-out") zoomStep(-1);
        if (action === "reset-selection") { state.selected = null; renderAll(); }
        if (action === "example") {
          try { applyPayload(validateAndFreezePayload(bundledExample(), "bundled example"), "bundled", false); } catch (error) { setNotice(error.message, "error"); }
        }
        if (action === "help" && typeof refs.keyboardDialog.showModal === "function") refs.keyboardDialog.showModal();
        if (action === "close-help") refs.keyboardDialog.close();
        if (action === "replay-prev" && state.replay.index > 0) { refs.replaySelect.value = String(state.replay.index - 1); loadReplayFrame(state.replay.index - 1); }
        if (action === "replay-next" && state.replay.index + 1 < state.replay.frames.length) { refs.replaySelect.value = String(state.replay.index + 1); loadReplayFrame(state.replay.index + 1); }
      });
    });
    document.querySelectorAll("[data-overlay]").forEach(function (button) {
      button.addEventListener("click", function () { var name = button.getAttribute("data-overlay"); state.overlays[name] = !state.overlays[name]; renderAll(); });
    });
    refs.artifactFile.addEventListener("change", function () { if (refs.artifactFile.files && refs.artifactFile.files[0]) readFile(refs.artifactFile.files[0]); refs.artifactFile.value = ""; });
    refs.search.addEventListener("input", function () { state.search = refs.search.value; renderSearchResults(); renderGraph(); });
    refs.search.addEventListener("keydown", function (event) {
      if (event.key === "Enter") {
        var first = state.payload && state.payload.map.nodes.find(function (node) { return node.id.toLowerCase().includes(state.search.trim().toLowerCase()) || state.payload.aliases[node.id].toLowerCase().includes(state.search.trim().toLowerCase()); });
        if (first) { refs.searchResults.hidden = true; selectNode(first.id, true); }
      }
      if (event.key === "Escape") { refs.search.value = ""; state.search = ""; refs.searchResults.hidden = true; }
    });
    refs.replaySelect.addEventListener("change", function () { if (refs.replaySelect.value === "") { leaveReplay(); return; } loadReplayFrame(Number(refs.replaySelect.value)); });
    refs.mapSvg.addEventListener("wheel", function (event) { event.preventDefault(); zoomAt(state.zoom * (event.deltaY < 0 ? 1.12 : 0.89), event.clientX, event.clientY); }, { passive: false });
    refs.mapSvg.addEventListener("pointerdown", function (event) { if (event.button !== 0 && event.pointerType !== "touch") return; state.drag.active = true; state.drag.moved = false; state.drag.x = event.clientX; state.drag.y = event.clientY; refs.mapSvg.classList.add("is-dragging"); refs.mapSvg.setPointerCapture(event.pointerId); });
    refs.mapSvg.addEventListener("pointermove", function (event) { if (!state.drag.active) return; var rect = refs.mapSvg.getBoundingClientRect(); var dx = event.clientX - state.drag.x; var dy = event.clientY - state.drag.y; if (Math.abs(dx) + Math.abs(dy) > 3) state.drag.moved = true; state.pan.x += dx / Math.max(1, rect.width) * viewWidth(); state.pan.y += dy / Math.max(1, rect.height) * VIEWBOX.height; state.drag.x = event.clientX; state.drag.y = event.clientY; applyTransform(); });
    refs.mapSvg.addEventListener("pointerup", function (event) { state.drag.active = false; refs.mapSvg.classList.remove("is-dragging"); if (refs.mapSvg.hasPointerCapture(event.pointerId)) refs.mapSvg.releasePointerCapture(event.pointerId); window.setTimeout(function () { state.drag.moved = false; }, 0); });
    refs.mapSvg.addEventListener("pointercancel", function () { state.drag.active = false; refs.mapSvg.classList.remove("is-dragging"); });
    document.addEventListener("keydown", function (event) {
      var target = event.target;
      var editing = target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT" || target.isContentEditable);
      if (event.key === "/" && !editing) { event.preventDefault(); refs.search.focus(); return; }
      if (editing) return;
      if (event.key === "0") { event.preventDefault(); fitMap(); }
      if (event.key === "+" || event.key === "=") { event.preventDefault(); zoomStep(1); }
      if (event.key === "-") { event.preventDefault(); zoomStep(-1); }
      if (event.key === "Escape") { state.selected = null; refs.search.value = ""; state.search = ""; renderAll(); }
    });
    refs.mapStage.addEventListener("click", function (event) { if (event.target === refs.mapSvg || event.target === byId("map-background")) { state.selected = null; renderAll(); } });
    window.addEventListener("resize", function () {
      if (!state.payload || !state.layout) return;
      var compact = refs.mapStage.clientWidth < 560;
      if (compact === state.layout.compact) return;
      state.layout = createLayout(state.payload);
      updateMapViewBox();
      renderAll();
    });
  }

  function startLive() {
    if (window.location.protocol !== "http:" && window.location.protocol !== "https:") {
      renderAll();
      return;
    }
    state.live.enabled = true;
    pollCurrent();
    loadReplayIndex();
    state.live.timer = window.setInterval(pollCurrent, POLL_MS);
  }

  function boot() {
    initReferences();
    initEvents();
    renderAll();
    startLive();
    window.__ASCENSION_MAP_VIEWER__ = { validatePayload: validatePayload, parseJsonBuffer: parseJsonBuffer, getState: function () { return state; } };
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", boot, { once: true });
  else boot();
}());
