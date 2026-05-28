import queryString from "node:querystring";
import type {Component} from "common/src/Component.ts";

export const componentsCache = new Map<string, Component>();

export const canvas = document.getElementById("canvas");
export const propertiesBody = document.getElementById("properties-body");

export let query_cache: Record<string, string | string[]> | null = null;

export function get_query(): Record<string, string | string[]> {
    if (!query_cache) {
        query_cache = queryString.parse(location.search.slice(1, location.search.length)) as NodeJS.Dict<string | string[]> as Record<string, string | string[]>;
    }
    return query_cache;
}