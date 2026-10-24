"use strict";
globalThis.SpireReportDom=Object.freeze({
  text(tag,value,className){
    const node=document.createElement(tag);node.textContent=value===null||value===undefined?"Unknown":String(value);
    if(className)node.className=className;return node;
  },
  asset(report,key,alt,className){
    const label=SpireReportDom.text("span",alt,"art-label");
    const byKey=report.assets?.byKey;const blobs=report.assets?.blobs;
    const blob=byKey&&Object.hasOwn(byKey,key)?byKey[key]:null;
    const url=blobs&&Object.hasOwn(blobs,blob)?blobs[blob]:null;
    if(typeof url!=="string"||!/^data:image\/(png|jpeg|webp);base64,[A-Za-z0-9+/]+={0,2}$/.test(url))return label;
    const image=document.createElement("img");image.alt=alt;image.className=className||"art";
    image.addEventListener("error",()=>image.replaceWith(label),{once:true});image.src=url;return image;
  },
  replaceChildren(parent,...nodes){parent.replaceChildren(...nodes);return parent;},
});
