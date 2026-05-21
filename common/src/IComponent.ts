import type {
    IComponentFetchDataContext, IComponentInitPropsContext, IComponentMountContext,
    IComponentNormalizeContext, IComponentSetSizeContext
} from "./contexts.ts";

export interface IComponentHooks {
    fetchData?: (ctx: IComponentFetchDataContext) => Record<string, string>;
    initProps?: (ctx: IComponentInitPropsContext) => void;
    setSize?: (ctx: IComponentSetSizeContext) => void;
    normalize?: (ctx: IComponentNormalizeContext) => void;
    mount?: (ctx: IComponentMountContext) => void;
}

export interface IComponent {
    name: string;
    hooks: IComponentHooks;
    title: string;
    html: string;
    data: Record<string, string>;
}