import queryString from "node:querystring";
import type {IComponent} from "common/src/IComponent.ts";

export type ComponentInfo = {
    id: string;
    path: string;
    html: string;
    title: string;
    data: Record<string, any>;
};

export const componentsRegistry = new Map<string, IComponent>();

export const canvas = document.getElementById("canvas");
export const propertiesBody = document.getElementById("properties-body");

export let query_cache: Record<string, string | string[]> | null = null;

export function get_query(): Record<string, string | string[]> {
    if (!query_cache) {
        query_cache = queryString.parse(location.search.slice(1, location.search.length)) as NodeJS.Dict<string | string[]> as Record<string, string | string[]>;
    }
    return query_cache;
}