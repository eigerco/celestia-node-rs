Error.stackTraceLimit = 99; // rust stack traces can get pretty big, increase the default

import { AppVersion, Blob, Namespace, NodeConfig, TxClient, protoEncodeSignDoc, spawnNode } from "lumina-node";
import { secp256k1 } from "@noble/curves/secp256k1";
import { Registry } from "@cosmjs/proto-signing";


// Expose classes on window so they can be used from the console
window.AppVersion = AppVersion;
window.Blob = Blob;
window.Namespace = Namespace;

// cat ci/credentials/bridge-0.address
window.bridge0Address = "celestia1t52q7uqgnjfzdh3wx5m5phvma3umrq8k6tq2p9";

async function createTxClient() {
  // cat ci/credentials/bridge-0.plaintext-key
  const privKey = "393fdb5def075819de55756b45c9e2c8531a8c78dd6eede483d3440e9457d839";
  const pubKey = secp256k1.getPublicKey(privKey);

  const signer = (signDoc) => {
    const bytes = protoEncodeSignDoc(signDoc);
    const sig = secp256k1.sign(bytes, privKey, { prehash: true });
    return sig.toCompactRawBytes();
  };

  const txClient = await new TxClient(
    "http://127.0.0.1:18080",
    window.bridge0Address,
    pubKey,
    signer
  );
  return txClient;
}

async function submitBankMsgSend(address, amount) {
  const registry = new Registry();
  const sendMsg = {
    typeUrl: "/cosmos.bank.v1beta1.MsgSend",
    value: {
      fromAddress: window.bridge0Address,
      toAddress: address,
      amount: [{ denom: "utia", amount: amount.toString() }],
    },
  };
  const sendMsgAny = registry.encodeAsAny(sendMsg);
  const txInfo = await window.txClient.submitMessage(sendMsgAny);

  return txInfo;
}

async function showStats(node) {
  if (!node || !await node.isRunning()) {
    return;
  }
  const info = await node.syncerInfo();
  document.getElementById("stored-ranges").innerText = info.stored_headers.map((range) => {
    return `${range.start}..${range.end}`;
  }).join(", ");

  let peersUl = document.createElement('ul');
  (await node.connectedPeers()).forEach(peer => {
    var li = document.createElement("li");
    li.innerText = peer;
    li.classList.add("mono");
    peersUl.appendChild(li);
  });

  document.getElementById("peers").replaceChildren(peersUl);

  const networkHead = await node.getNetworkHeadHeader();
  if (networkHead == null) {
    return;
  }

  const squareRows = networkHead.dah.rowRoots().length;
  const squareCols = networkHead.dah.columnRoots().length;

  document.getElementById("block-height").innerText = networkHead.header.height;
  document.getElementById("block-hash").innerText = networkHead.commit.block_id.hash;
  document.getElementById("block-data-square").innerText = `${squareRows}x${squareCols} shares`;
}

function logEvent(event) {
  // Skip noisy events
  if (event.data.get("event").type == "share_sampling_result") {
    return;
  }

  const time = new Date(event.data.get("time"));

  const log = time.getHours().toString().padStart(2, '0')
    + ":" + time.getMinutes().toString().padStart(2, '0')
    + ":" + time.getSeconds().toString().padStart(2, '0')
    + "." + time.getMilliseconds().toString().padStart(3, '0')
    + ": " + event.data.get("message");

  var textarea = document.getElementById("event-logs");
  textarea.value += log + "\n";
  textarea.scrollTop = textarea.scrollHeight;
}

function starting(document) {
  document.getElementById("start-stop").disabled = true;
  document.querySelectorAll('.config').forEach(elem => elem.disabled = true);
}

async function started(document, window) {
  document.getElementById("peer-id").innerText = await window.node.localPeerId();
  document.querySelectorAll(".status").forEach(elem => elem.style.visibility = "visible");
  document.getElementById("start-stop").innerText = "Stop";
  document.getElementById("start-stop").disabled = false;
  window.showStatsIntervalId = setInterval(async () => await showStats(window.node), 1000);
}

function stopping(document, window) {
  clearInterval(window.showStatsIntervalId);
  document.getElementById("start-stop").disabled = true;
}

function stopped(document) {
  document.querySelectorAll(".status").forEach(elem => elem.style.visibility = "hidden");
  document.querySelectorAll(".status-value").forEach(elem => elem.innerText = "");
  document.getElementById("start-stop").innerText = "Start";
  document.querySelectorAll('.config').forEach(elem => elem.disabled = false);
  document.getElementById("start-stop").disabled = false;
}

async function main(document, window) {
  window.node = await spawnNode();
  window.txClient = await createTxClient();

  window.events = await window.node.eventsChannel();
  window.events.onmessage = (event) => {
    logEvent(event);
  };

  const networkIdDiv = document.getElementById("network-id");
  const bootnodesDiv = document.getElementById("bootnodes");
  const startStopDiv = document.getElementById("start-stop");

  window.config = NodeConfig.default(0);
  bootnodesDiv.value = window.config.bootnodes.join("\n");

  networkIdDiv.addEventListener("change", event => {
    window.config = NodeConfig.default(Number(event.target.value));
    bootnodesDiv.value = window.config.bootnodes.join("\n");
  });

  bootnodesDiv.addEventListener("change", event => {
    window.config.bootnodes = event.target.value.trim().split("\n").map(multiaddr => multiaddr.trim());
  });

  startStopDiv.addEventListener("click", async () => {
    if (await window.node.isRunning() === true) {
      stopping(document, window);
      await window.node.stop();
      stopped(document);
    } else {
      starting(document);
      await window.node.start(window.config);
      await started(document, window);
    }
  });

  // test submitting transfer
  const txInfo = await submitBankMsgSend(window.bridge0Address, 10000);
  console.log("Submitting bank MsgSend successful", txInfo);
}

await main(document, window);
