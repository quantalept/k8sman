/**
 * Shared max-height for NDataTable instances, so only the row content scrolls internally -
 * the header, filters/search row, and page chrome above the table stay put. Tuned for the
 * page header (48px) + content padding (48px) + a card title + one filter row; nested-in-tab
 * tables have less chrome above them so get a shorter offset.
 */
export const PAGE_TABLE_MAX_HEIGHT = "calc(100vh - 320px)";
export const TAB_TABLE_MAX_HEIGHT = "calc(100vh - 260px)";
