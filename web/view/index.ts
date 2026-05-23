import {
    ComponentMountContext,
    ComponentNormalizeContext,
    ComponentSetSizeContext
} from "editor/src/component-contexts.ts";
import {type ComponentInfo, componentsRegistry} from "editor/src/common.ts";
import type {IComponent} from "common/src/IComponent.ts";
import type {ComponentHooks} from "editor/src/import-components.ts";

declare const components: ComponentInfo[];

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

    console.log("components loaded, starting view systems");

    document.querySelectorAll('[data-type]').forEach(e => {
        const element = e as HTMLElement;
        const type = element.getAttribute('data-type');
        const filtered = Object.entries(element.dataset).filter(entry => entry[0] !== 'type').filter((entry): entry is [string, string] => entry[1] !== undefined);
        const data = Object.fromEntries(filtered);

        if (!type) {
            console.error('No type attribute found on element', element);
            return;
        }

        const component = componentsRegistry.get(type);
        if (!component) {
            console.error('No component found for type', type);
            return;
        }

        const compData = {...component.data, ...data};

        if (component.hooks.mount) {
            const mountCtx = new ComponentMountContext(element, component.html, compData, component.title);
            component.hooks.mount(mountCtx);
        } else {
            console.error('No mount hook found for component', type);
        }

        if (component.hooks.setSize) {
            const sizeCtx = new ComponentSetSizeContext(element, compData);
            component.hooks.setSize(sizeCtx);
        } else {
            console.error('No size hook found for component', type);
        }
    });
})