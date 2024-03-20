import { parse } from "$std/flags/mod.ts";
import { Builder, BuilderOptions } from "https://raw.githubusercontent.com/maemon4095/tauri-deno-builder/release/v0.1.0/src/mod.ts";

const args = parse(Deno.args, {
    boolean: ["dev"],
});
const is_dev = args.dev;
const mode = args._[0];

const commonOptions: BuilderOptions = {
    documentFilePath: "./index.html",
    denoConfigPath: "./deno.json",
    esbuildOptions: {
        jsxFactory: "h",
        jsxFragment: "Fragment"
    }
};

const options: BuilderOptions = is_dev ? commonOptions : {
    ...commonOptions,
    minifySyntax: true,
    dropLabels: ["DEV"]
};

const builder = new Builder(options);

switch (mode) {
    case "serve": {
        await builder.serve();
        break;
    }
    case "build": {
        await builder.build();
        break;
    }
}