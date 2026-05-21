import {type ComponentInfo, componentsRegistry} from "./common.ts";
import type {IComponent} from "common/src/IComponent.ts";
import {initDrag} from "./drag-n-drop.ts";
import {normalizeComponents} from "./normilize-components.ts";
import {initDeleteButton} from "./delete-btn.ts";

declare const components: ComponentInfo[];

for (const comp of components) {
    import(comp.path).then(componentHooks => {
        console.log("imported (" + comp.id + "): ", componentHooks);
        const component: IComponent = {
            hooks: componentHooks.hooks,
            html: comp.html,
            title: comp.title,
            data: comp.data,
        };
        componentsRegistry.set(comp.id, component);
    }).then(() => {
        console.log("all components imported, init other systems");

        initDrag();
        initDeleteButton();
        document.querySelectorAll(".grid-block").forEach(el => normalizeComponents(el as HTMLDivElement));
    });
}