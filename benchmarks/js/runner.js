var ROOT = typeof globalThis !== "undefined" ? globalThis : this;

function writeLine(line) {
  if (typeof console !== "undefined" && console.log) {
    console.log(line);
  } else if (typeof print !== "undefined") {
    print(line);
  }
}

function mix(left, right) {
  var x = (left ^ Math.imul(right, 0x9e3779b1)) >>> 0;
  x ^= x >>> 16;
  x = Math.imul(x, 0x85ebca6b) >>> 0;
  x ^= x >>> 13;
  x = Math.imul(x, 0xc2b2ae35) >>> 0;
  return (x ^ (x >>> 16)) >>> 0;
}

function rotl(value, shift) {
  return ((value << shift) | (value >>> (32 - shift))) >>> 0;
}

function parseArgs() {
  var raw = [];
  if (typeof scriptArgs !== "undefined") {
    raw = scriptArgs.slice(1);
  } else if (typeof process !== "undefined" && process.argv) {
    raw = process.argv.slice(2);
  }

  var options = { samples: 5, scale: 1, caseFilter: null };
  for (var i = 0; i < raw.length; i++) {
    if (raw[i] === "--samples" && i + 1 < raw.length) {
      options.samples = Math.max(1, parseInt(raw[++i], 10) || options.samples);
    } else if (raw[i] === "--scale" && i + 1 < raw.length) {
      options.scale = Math.max(1, parseInt(raw[++i], 10) || options.scale);
    } else if (raw[i] === "--case" && i + 1 < raw.length) {
      options.caseFilter = String(raw[++i]).toLowerCase();
    }
  }
  return options;
}

function richards(iterations) {
  var taskCount = 6;
  var queues = [];
  var state = [];
  for (var task = 0; task < taskCount; task++) {
    queues.push([{ target: (task + 1) % taskCount, value: mix(task, 1) }]);
    state.push(0);
  }

  for (var tick = 0; tick < iterations; tick++) {
    for (var id = 0; id < taskCount; id++) {
      queues[id].push({
        target: (id * 3 + tick + 1) % taskCount,
        value: mix(tick, id)
      });

      var budget = 3 + ((tick + id) % 5);
      for (var count = 0; count < budget; count++) {
        if (queues[id].length === 0) {
          break;
        }
        var packet = queues[id].shift();
        var value = mix((packet.value ^ state[id]) >>> 0, tick + id);
        state[id] = (rotl(state[id], 7) ^ value) >>> 0;
        var next = (packet.target + (value & 3) + 1) % taskCount;
        if (next !== id) {
          queues[next].push({ target: id, value: value });
        }
      }
    }
  }

  var checksum = 0;
  for (var queueIndex = 0; queueIndex < queues.length; queueIndex++) {
    var queueSum = queues[queueIndex].length >>> 0;
    for (var packetIndex = 0; packetIndex < Math.min(16, queues[queueIndex].length); packetIndex++) {
      queueSum = (queueSum ^ queues[queueIndex][packetIndex].value) >>> 0;
    }
    checksum = mix((checksum ^ state[queueIndex]) >>> 0, queueSum);
  }
  return checksum >>> 0;
}

function deltaBlue(iterations) {
  var values = [];
  var constraints = [];
  for (var i = 0; i < 96; i++) {
    values.push((i + 1) * 0.25);
  }
  for (var index = 0; index < 192; index++) {
    constraints.push({
      left: index % values.length,
      right: (index * 7 + 13) % values.length,
      output: (index * 11 + 17) % values.length,
      scale: 0.875 + (index % 9) * 0.03125,
      bias: (index % 5) - 2.0
    });
  }

  for (var step = 0; step < iterations; step++) {
    var anchor = step % values.length;
    values[0] = (step + 1) * 0.125;
    values[anchor] = values[anchor] * 0.5 + step * 0.015625;

    for (var pass = 0; pass < 3; pass++) {
      for (var c = 0; c < constraints.length; c++) {
        var constraint = constraints[c];
        var propagated = values[constraint.left] * constraint.scale +
          values[constraint.right] * 0.125 +
          constraint.bias;
        values[constraint.output] = propagated * 0.999 + constraint.bias * 0.001;
      }
    }
  }

  var checksum = 0;
  for (var valueIndex = 0; valueIndex < values.length; valueIndex++) {
    checksum = mix(checksum, ((values[valueIndex] * 1000003) | 0) ^ valueIndex);
  }
  return checksum >>> 0;
}

function crypto(iterations) {
  var data = [];
  for (var index = 0; index < 4096; index++) {
    data.push(mix(index, 0x51ed270b) & 255);
  }
  var state = 0x243f6a88 >>> 0;

  for (var round = 0; round < iterations * 12; round++) {
    for (var offset = 0; offset < data.length; offset += 4) {
      var word = (data[offset] |
        (data[offset + 1] << 8) |
        (data[offset + 2] << 16) |
        (data[offset + 3] << 24)) >>> 0;
      state = mix(rotl(state, 9), (word ^ round) >>> 0);
      data[offset] = state & 255;
      data[offset + 1] = (state >>> 8) & 255;
      data[offset + 2] = (state >>> 16) & 255;
      data[offset + 3] = (state >>> 24) & 255;
    }

    var rotation = (round % (data.length - 1)) + 1;
    data = data.slice(rotation).concat(data.slice(0, rotation));
  }

  var checksum = state;
  for (var i = 0; i < data.length; i += 4) {
    var chunk = (data[i] | (data[i + 1] << 8) | (data[i + 2] << 16) | (data[i + 3] << 24)) >>> 0;
    checksum = mix((checksum ^ (i >>> 2)) >>> 0, chunk);
  }
  return checksum >>> 0;
}

function vec3(x, y, z) {
  return { x: x, y: y, z: z };
}

function vadd(a, b) {
  return vec3(a.x + b.x, a.y + b.y, a.z + b.z);
}

function vsub(a, b) {
  return vec3(a.x - b.x, a.y - b.y, a.z - b.z);
}

function vscale(a, factor) {
  return vec3(a.x * factor, a.y * factor, a.z * factor);
}

function vdot(a, b) {
  return a.x * b.x + a.y * b.y + a.z * b.z;
}

function vnormalize(a) {
  var length = Math.sqrt(vdot(a, a));
  return length === 0 ? a : vscale(a, 1 / length);
}

function intersectSphere(origin, direction, sphere) {
  var oc = vsub(origin, sphere.center);
  var b = vdot(oc, direction);
  var c = vdot(oc, oc) - sphere.radius * sphere.radius;
  var discriminant = b * b - c;
  if (discriminant < 0) {
    return null;
  }
  var distance = -b - Math.sqrt(discriminant);
  return distance > 0.001 ? distance : null;
}

function rayTrace(iterations) {
  var spheres = [
    { center: vec3(-1.25, -0.2, 3.6), radius: 0.7, albedo: 0.85 },
    { center: vec3(0.85, 0.0, 3.0), radius: 0.55, albedo: 0.75 },
    { center: vec3(0.15, -0.8, 4.25), radius: 0.9, albedo: 0.65 },
    { center: vec3(1.6, 0.35, 4.6), radius: 0.5, albedo: 0.9 }
  ];
  var checksum = 0;

  for (var frame = 0; frame < iterations; frame++) {
    var camera = vec3(Math.sin(frame * 0.011) * 0.25, 0.15, -2.75);
    for (var y = 0; y < 28; y++) {
      for (var x = 0; x < 36; x++) {
        var origin = camera;
        var direction = vnormalize(vec3((x - 18.0) / 22.0, (14.0 - y) / 22.0, 1.0));
        var light = 0;
        var throughput = 1;

        for (var bounce = 0; bounce < 2; bounce++) {
          var closestDistance = Infinity;
          var closestSphere = null;
          for (var s = 0; s < spheres.length; s++) {
            var distance = intersectSphere(origin, direction, spheres[s]);
            if (distance !== null && distance < closestDistance) {
              closestDistance = distance;
              closestSphere = spheres[s];
            }
          }

          if (closestSphere === null) {
            light += throughput * (0.2 + 0.03 * y);
            break;
          }

          var hit = vadd(origin, vscale(direction, closestDistance));
          var normal = vnormalize(vsub(hit, closestSphere.center));
          var lambert = Math.max(0, vdot(normal, vnormalize(vec3(-0.4, 0.9, -0.3))));
          light += throughput * closestSphere.albedo * lambert;
          throughput *= 0.42 + bounce * 0.08;
          origin = vadd(hit, vscale(normal, 0.001));
          direction = vnormalize(vsub(direction, vscale(normal, 2 * vdot(direction, normal))));
        }

        checksum = mix(checksum, ((light * 1000003) | 0) ^ ((frame << 16) + y * 36 + x));
      }
    }
  }

  return checksum >>> 0;
}

var NT_START = 0;
var NT_EXPR = 1;
var NT_TERM = 2;
var NT_FACTOR = 3;
var TOK_NUM = 1;
var TOK_PLUS = 2;
var TOK_STAR = 3;
var TOK_LPAREN = 4;
var TOK_RPAREN = 5;

var GRAMMAR = [
  { lhs: NT_START, rhs: [["n", NT_EXPR]] },
  { lhs: NT_EXPR, rhs: [["n", NT_EXPR], ["t", TOK_PLUS], ["n", NT_TERM]] },
  { lhs: NT_EXPR, rhs: [["n", NT_TERM]] },
  { lhs: NT_TERM, rhs: [["n", NT_TERM], ["t", TOK_STAR], ["n", NT_FACTOR]] },
  { lhs: NT_TERM, rhs: [["n", NT_FACTOR]] },
  { lhs: NT_FACTOR, rhs: [["t", TOK_LPAREN], ["n", NT_EXPR], ["t", TOK_RPAREN]] },
  { lhs: NT_FACTOR, rhs: [["t", TOK_NUM]] }
];

function stateKey(state) {
  return state.rule + ":" + state.dot + ":" + state.origin;
}

function addState(chartEntry, state) {
  var key = stateKey(state);
  if (!chartEntry.map[key]) {
    chartEntry.map[key] = true;
    chartEntry.list.push(state);
  }
}

function expressionTokens(seed, terms) {
  var tokens = [];
  for (var term = 0; term < terms; term++) {
    if (term % 4 === 0 && term + 1 < terms) {
      tokens.push(TOK_LPAREN, TOK_NUM, TOK_PLUS, TOK_NUM, TOK_RPAREN);
    } else {
      tokens.push(TOK_NUM);
    }
    if (term + 1 < terms) {
      tokens.push((term + seed) % 3 === 0 ? TOK_STAR : TOK_PLUS);
    }
  }
  return tokens;
}

function earleyParse(tokens) {
  var chart = [];
  for (var i = 0; i <= tokens.length; i++) {
    chart.push({ list: [], map: {} });
  }
  addState(chart[0], { rule: 0, dot: 0, origin: 0 });

  for (var index = 0; index <= tokens.length; index++) {
    for (var cursor = 0; cursor < chart[index].list.length; cursor++) {
      var state = chart[index].list[cursor];
      var rule = GRAMMAR[state.rule];
      var symbol = rule.rhs[state.dot];

      if (symbol && symbol[0] === "n") {
        for (var ruleIndex = 0; ruleIndex < GRAMMAR.length; ruleIndex++) {
          if (GRAMMAR[ruleIndex].lhs === symbol[1]) {
            addState(chart[index], { rule: ruleIndex, dot: 0, origin: index });
          }
        }
      } else if (symbol && symbol[0] === "t") {
        if (tokens[index] === symbol[1]) {
          addState(chart[index + 1], { rule: state.rule, dot: state.dot + 1, origin: state.origin });
        }
      } else {
        var completedLhs = rule.lhs;
        var originStates = chart[state.origin].list;
        for (var j = 0; j < originStates.length; j++) {
          var previous = originStates[j];
          var previousSymbol = GRAMMAR[previous.rule].rhs[previous.dot];
          if (previousSymbol && previousSymbol[0] === "n" && previousSymbol[1] === completedLhs) {
            addState(chart[index], {
              rule: previous.rule,
              dot: previous.dot + 1,
              origin: previous.origin
            });
          }
        }
      }
    }
  }

  var accepted = !!chart[tokens.length].map["0:1:0"];
  var totalStates = 0;
  for (var entry = 0; entry < chart.length; entry++) {
    totalStates += chart[entry].list.length;
  }
  return { accepted: accepted, states: totalStates };
}

function boyerRewrite(seed) {
  var terms = [];
  for (var index = 0; index < 48; index++) {
    var lhs = mix(seed, index);
    var rhs = mix(index, (seed ^ 0x0badf00d) >>> 0);
    terms.push({ left: lhs & 31, right: rhs & 31, value: mix(lhs, rhs) });
  }

  for (var round = 0; round < 16; round++) {
    terms.sort(function (a, b) {
      return (a.left - b.left) || (a.right - b.right) || (a.value - b.value);
    });

    var deduped = [];
    var previousKey = null;
    for (var i = 0; i < terms.length; i++) {
      var key = terms[i].left + ":" + terms[i].right;
      if (key !== previousKey) {
        deduped.push(terms[i]);
        previousKey = key;
      }
    }
    terms = deduped;

    for (var j = 0; j < terms.length; j++) {
      var term = terms[j];
      if (term.left === term.right) {
        term.value = mix(term.value, round);
      } else {
        term.left = (term.left + term.value + round) & 31;
        term.right = (term.right ^ (term.value >>> 3)) & 31;
        term.value = mix(term.value, (term.left ^ term.right ^ round) >>> 0);
      }
    }

    while (terms.length < 48) {
      var next = mix((seed ^ round) >>> 0, terms.length);
      terms.push({ left: next & 31, right: (next >>> 7) & 31, value: next });
    }
  }

  var checksum = seed >>> 0;
  for (var t = 0; t < terms.length; t++) {
    checksum = mix((checksum ^ terms[t].left) >>> 0, (terms[t].right ^ terms[t].value) >>> 0);
  }
  return checksum >>> 0;
}

function earleyBoyer(iterations) {
  var checksum = 0;
  for (var seed = 0; seed < iterations; seed++) {
    var tokens = expressionTokens(seed, 8 + (seed % 5));
    var parsed = earleyParse(tokens);
    var rewriteSum = boyerRewrite(seed);
    checksum = mix((checksum ^ rewriteSum) >>> 0, (parsed.states ^ (parsed.accepted ? 1 : 0) ^ tokens.length) >>> 0);
  }
  return checksum >>> 0;
}

function countOverlapping(haystack, needle) {
  var count = 0;
  var start = 0;
  while (start < haystack.length) {
    var position = haystack.indexOf(needle, start);
    if (position < 0) {
      break;
    }
    count++;
    start = position + 1;
  }
  return count;
}

function regexp(iterations) {
  var patterns = ["agggtaaa", "tttaccct", "cgggtaaa", "gggtaaat", "taaaacc", "gtaac"];
  var alphabet = ["a", "c", "g", "t"];
  var dna = "";
  for (var index = 0; index < 8192; index++) {
    dna += alphabet[mix(index, 17) & 3];
  }

  var checksum = 0;
  for (var iteration = 0; iteration < iterations; iteration++) {
    if (iteration % 8 === 0) {
      dna = dna.split("").reverse().join("");
    }

    for (var p = 0; p < patterns.length; p++) {
      checksum = mix(checksum, countOverlapping(dna, patterns[p]));
    }

    var gc = 0;
    for (var i = 0; i < dna.length; i++) {
      var ch = dna.charAt(i);
      if (ch === "g" || ch === "c") {
        gc++;
      }
    }
    var runs = 0;
    var segments = dna.split("a");
    for (var s = 0; s < segments.length; s++) {
      if (segments[s].length > 0) {
        runs++;
      }
    }
    checksum = mix((checksum ^ gc) >>> 0, (runs ^ iteration) >>> 0);
  }
  return checksum >>> 0;
}

function splay(iterations) {
  var map = {};
  var keys = [];
  for (var key = 0; key < 1024; key++) {
    var stored = mix(key, 0xfeedface) & 4095;
    if (map[stored] === undefined) {
      keys.push(stored);
    }
    map[stored] = mix(key, 0x12345678);
  }

  var checksum = 0;
  for (var step = 0; step < iterations * 24; step++) {
    var nextKey = mix(step, checksum) & 4095;
    if (map[nextKey] === undefined) {
      keys.push(nextKey);
    }
    map[nextKey] = mix(nextKey, step);

    if (step % 3 === 0) {
      var removeKey = mix(step, 7) & 4095;
      if (map[removeKey] !== undefined) {
        delete map[removeKey];
      }
    }

    if (step % 8 === 0) {
      keys = keys.filter(function (candidate) { return map[candidate] !== undefined; });
      keys.sort(function (a, b) { return a - b; });
      var lower = Math.max(0, nextKey - 24);
      var upper = nextKey + 24;
      var consumed = 0;
      for (var i = 0; i < keys.length && consumed < 12; i++) {
        if (keys[i] >= lower && keys[i] <= upper) {
          checksum = mix((checksum ^ keys[i]) >>> 0, map[keys[i]]);
          consumed++;
        }
      }
    }
  }

  keys = keys.filter(function (candidate) { return map[candidate] !== undefined; });
  keys.sort(function (a, b) { return a - b; });
  var finalChecksum = mix(checksum, keys.length);
  for (var index = 0; index < Math.min(256, keys.length); index++) {
    finalChecksum = mix((finalChecksum ^ keys[index]) >>> 0, map[keys[index]]);
  }
  return finalChecksum >>> 0;
}

function fieldIndex(n, x, y) {
  return y * (n + 2) + x;
}

function bilinearSample(field, n, x, y) {
  var x0 = Math.floor(x);
  var y0 = Math.floor(y);
  var x1 = Math.min(x0 + 1, n);
  var y1 = Math.min(y0 + 1, n);
  var sx = x - x0;
  var sy = y - y0;
  var top = field[fieldIndex(n, x0, y0)] * (1 - sx) + field[fieldIndex(n, x1, y0)] * sx;
  var bottom = field[fieldIndex(n, x0, y1)] * (1 - sx) + field[fieldIndex(n, x1, y1)] * sx;
  return top * (1 - sy) + bottom * sy;
}

function navierStokes(iterations) {
  var n = 32;
  var size = (n + 2) * (n + 2);
  var density = [];
  var velocityX = [];
  var velocityY = [];
  for (var i = 0; i < size; i++) {
    density.push(0);
    velocityX.push(0);
    velocityY.push(0);
  }

  for (var step = 0; step < iterations; step++) {
    var centerX = 8 + (step % 17);
    var centerY = 8 + ((step * 5) % 17);
    var center = fieldIndex(n, centerX, centerY);
    density[center] += 24 + (step % 7);
    velocityX[center] += Math.sin(step * 0.13) * 0.7;
    velocityY[center] += Math.cos(step * 0.17) * 0.7;

    for (var pass = 0; pass < 4; pass++) {
      for (var y = 1; y <= n; y++) {
        for (var x = 1; x <= n; x++) {
          var current = fieldIndex(n, x, y);
          density[current] = (
            density[current] +
            density[fieldIndex(n, x - 1, y)] +
            density[fieldIndex(n, x + 1, y)] +
            density[fieldIndex(n, x, y - 1)] +
            density[fieldIndex(n, x, y + 1)]
          ) * 0.2;
          velocityX[current] = (
            velocityX[current] +
            velocityX[fieldIndex(n, x - 1, y)] +
            velocityX[fieldIndex(n, x + 1, y)]
          ) / 3;
          velocityY[current] = (
            velocityY[current] +
            velocityY[fieldIndex(n, x, y - 1)] +
            velocityY[fieldIndex(n, x, y + 1)]
          ) / 3;
        }
      }
    }

    var previous = density.slice();
    for (var yy = 1; yy <= n; yy++) {
      for (var xx = 1; xx <= n; xx++) {
        var index = fieldIndex(n, xx, yy);
        var sourceX = Math.max(1, Math.min(n, xx - velocityX[index]));
        var sourceY = Math.max(1, Math.min(n, yy - velocityY[index]));
        density[index] = bilinearSample(previous, n, sourceX, sourceY);
      }
    }
  }

  var checksum = 0;
  for (var cell = 0; cell < density.length; cell++) {
    checksum = mix((checksum ^ cell) >>> 0, (density[cell] * 1000003) | 0);
  }
  return checksum >>> 0;
}

var CASES = [
  { name: "Richards", iterations: 2400, run: richards },
  { name: "DeltaBlue", iterations: 700, run: deltaBlue },
  { name: "Crypto", iterations: 160, run: crypto },
  { name: "RayTrace", iterations: 26, run: rayTrace },
  { name: "EarleyBoyer", iterations: 36, run: earleyBoyer },
  { name: "RegExp", iterations: 180, run: regexp },
  { name: "Splay", iterations: 520, run: splay },
  { name: "NavierStokes", iterations: 34, run: navierStokes }
];

function nowMs() {
  return Date.now();
}

function main() {
  var options = parseArgs();
  writeLine(JSON.stringify({
    event: "suite",
    language: "javascript",
    target_arch: "current",
    samples: options.samples,
    scale: options.scale
  }));

  for (var i = 0; i < CASES.length; i++) {
    var benchCase = CASES[i];
    if (options.caseFilter && benchCase.name.toLowerCase() !== options.caseFilter) {
      continue;
    }

    var iterations = benchCase.iterations * options.scale;
    for (var sample = 0; sample < options.samples; sample++) {
      var start = nowMs();
      var checksum = benchCase.run(iterations);
      var elapsedMs = nowMs() - start;
      writeLine(JSON.stringify({
        event: "sample",
        case: benchCase.name,
        sample: sample + 1,
        iterations: iterations,
        elapsed_ms: elapsedMs,
        checksum: checksum
      }));
    }
  }
}

main();

