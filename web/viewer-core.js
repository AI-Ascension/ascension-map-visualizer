/* SPDX-License-Identifier: MIT */
// Shared viewer foundation: bounded constants, mutable session state, DOM/SVG helpers.
// Loaded before viewer-validation.js and app.js as a classic script so the entrypoint
// keeps working from file:// as well as the embedded http server.
window.__ASCENSION_MAP_MODULES__ = window.__ASCENSION_MAP_MODULES__ || {};
window.__ASCENSION_MAP_MODULES__.core = (function () {
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
  var STATUS_AVAILABILITY = new Set(["AVAILABLE", "UNAVAILABLE", "NOT_OBSERVABLE", "UNSUPPORTED"]);
  var STATUS_COMPLETENESS = new Set(["COMPLETE", "INCOMPLETE", "UNKNOWN"]);
  var STATUS_FRESHNESS = new Set(["CURRENT", "HISTORICAL"]);
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

  return {
    SVG_NS: SVG_NS, VIEWBOX: VIEWBOX, COMPACT_VIEWBOX_WIDTH: COMPACT_VIEWBOX_WIDTH,
    MAX_JSON_BYTES: MAX_JSON_BYTES, MAX_TEXT_BYTES: MAX_TEXT_BYTES, MAX_NODES: MAX_NODES,
    MAX_EDGES: MAX_EDGES, MAX_CANDIDATES: MAX_CANDIDATES, MAX_REPLAY_FRAMES: MAX_REPLAY_FRAMES,
    MAX_DEPTH: MAX_DEPTH, POLL_MS: POLL_MS, ZOOM_MIN: ZOOM_MIN, ZOOM_MAX: ZOOM_MAX,
    STATUS_AVAILABILITY: STATUS_AVAILABILITY, STATUS_COMPLETENESS: STATUS_COMPLETENESS,
    STATUS_FRESHNESS: STATUS_FRESHNESS,
    refs: refs, state: state,
    byId: byId, make: make, makeSvg: makeSvg, removeChildren: removeChildren
  };
}());
