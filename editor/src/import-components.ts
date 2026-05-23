import {type ComponentInfo, componentsRegistry} from "./common.ts";
import type {IComponent, IComponentHooks} from "common/src/IComponent.ts";
import {initDrag} from "./drag-n-drop.ts";
import {normalizeComponents} from "./normilize-components.ts";
import {initDeleteButton} from "./delete-btn.ts";
import {markedDirty, saveContent, unmarkDirty} from "./save-content.ts";

declare const components: ComponentInfo[];

export type ComponentHooks = {
    hooks: IComponentHooks
};

const componentsPromises: Promise<IComponent>[] = [];

for (const comp of components) {
    componentsPromises.push(import(comp.path).then((componentInfo: ComponentHooks): IComponent => {
            return {
                ...componentInfo,
                name: comp.id,
                html: comp.html,
                title: comp.title,
                data: comp.data,
            }
        }
    ));
}

Promise.all(componentsPromises).then(components => {
    for (const comp of components) {
        componentsRegistry.set(comp.name, comp);
    }

    console.log("components loaded, starting editor systems");

    setInterval(() => {
        if (!markedDirty()) return;

        saveContent();
        unmarkDirty();
    }, 5000);

    initDrag();
    initDeleteButton();
    document.querySelectorAll(".grid-block").forEach(el => normalizeComponents(el as HTMLDivElement));
})