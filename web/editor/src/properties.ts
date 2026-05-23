import {componentsRegistry, propertiesBody} from "./common.ts";
import {ComponentInitPropsContext} from "./component-contexts.ts";
import type {IComponent} from "common/src/IComponent.ts";

export function initProperties(block: HTMLDivElement, comp: IComponent | null = null) {
    block.addEventListener("click", () => {
        document.querySelectorAll(".grid-block.selected")
            .forEach(el => el.classList.remove("selected"));

        block.classList.add("selected");

        if (!propertiesBody) {
            console.error("no properties body found");
            return;
        }

        propertiesBody.innerHTML = "";

        if (!comp) {
            const type = block.dataset.type;
            if (!type) {
                console.error("no block type found")
                return;
            }

            const component = componentsRegistry.get(type);
            if (!component) {
                console.error("no component found for type: " + block.dataset.type);
                return;
            }
            comp = component;
        }

        const ctx = new ComponentInitPropsContext(block, comp.data, propertiesBody as HTMLDivElement);

        if (comp.hooks.initProps) {
            comp.hooks.initProps(ctx);
        }

        /* GRID WIDTH */
        ctx.createNumber(
            "Width (columns)",
            block.dataset.col ?? "12",
            "1", "12",
            value => {
                block.dataset.col = value;
                block.style.gridColumn = `span ${value}`;
            }
        );

        /* GRID HEIGHT */
        ctx.createNumber(
            "Height (rows)",
            block.dataset.row ?? "1",
            String(1), "",
            value => ctx.applyRows(Number(value))
        );
    });
}