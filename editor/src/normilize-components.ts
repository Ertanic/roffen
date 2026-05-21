import {initProperties} from "./properties.ts";
import {ComponentNormalizeContext} from "./component-contexts.ts";
import {componentsRegistry} from "./common.ts";

export function normalizeComponents(block: HTMLDivElement) {
    const type = block.dataset.type;

    if (!type) {
        console.error("no component type found");
        return;
    }

    const comp = componentsRegistry.get(type);

    if (!comp) {
        console.error("no component found for type: " + type);
        return;
    }

    if (comp.hooks.normalize) {
        const ctx = new ComponentNormalizeContext(block, comp.data);
        comp.hooks.normalize(ctx);
    }

    initProperties(block, comp);
}