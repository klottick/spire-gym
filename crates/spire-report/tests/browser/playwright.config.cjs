const {defineConfig}=require('@playwright/test');
const path=require('node:path');
module.exports=defineConfig({
  testDir:__dirname,testMatch:'**/*.spec.cjs',workers:1,reporter:'list',timeout:10000,
  outputDir:path.resolve(__dirname,'../../../../.superpowers/sdd/2026-10-23-spire-gym-run-viewer/browser-results'),
  use:{browserName:'chromium',offline:true,serviceWorkers:'block',viewport:{width:1440,height:900}},
});
