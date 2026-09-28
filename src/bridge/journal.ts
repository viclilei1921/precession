import { invokeCommand } from './invoke';

/** 手记种类 */
export type JournalKind = 'diary' | 'spark' | 'writing';

/** 手记之间的流转 */
export type JournalLinkKind = 'spark_to_writing' | 'writing_to_diary';

/** 一条手记 */
export type JournalEntry = {
  id: string;
  kind: JournalKind;
  occurredAt: number;
  title: string;
  body: string;
  locked: boolean;
  highlight: boolean;
  memberIds: string[];
  tagIds: string[];
  placeIds: string[];
  createdAt: number;
  updatedAt: number;
};

/** 新建或修改手记 */
export type JournalEntryInput = {
  kind: JournalKind;
  occurredAt: number;
  title: string;
  body: string;
  locked: boolean;
  highlight: boolean;
  memberIds: string[];
  tagIds: string[];
  placeIds: string[];
};

/** 手记之间的流转 */
export type JournalLink = {
  id: string;
  fromId: string;
  toId: string;
  kind: JournalLinkKind;
  createdAt: number;
};

/** 手记引用的书摘或读书笔记 */
export type JournalCitation = {
  id: string;
  entryId: string;
  bookNoteId: string;
  createdAt: number;
};

/** 按种类和时间列出手记。from 含，to 不含。 */
export function journalEntryList(kind?: JournalKind, from?: number, to?: number) {
  return invokeCommand<JournalEntry[]>('journal_entry_list', { kind, from, to });
}

/** 读取一条手记 */
export function journalEntryGet(id: string) {
  return invokeCommand<JournalEntry>('journal_entry_get', { id });
}

/** 新建手记 */
export function journalEntryCreate(input: JournalEntryInput) {
  return invokeCommand<JournalEntry>('journal_entry_create', { input });
}

/** 修改手记 */
export function journalEntryUpdate(id: string, input: JournalEntryInput) {
  return invokeCommand<JournalEntry>('journal_entry_update', { id, input });
}

/** 软删除手记 */
export function journalEntryDelete(id: string) {
  return invokeCommand<void>('journal_entry_delete', { id });
}

/** 列出和某条手记相关的流转 */
export function journalLinkList(entryId: string) {
  return invokeCommand<JournalLink[]>('journal_link_list', { entryId });
}

/** 建立手记流转 */
export function journalLinkCreate(fromId: string, toId: string, kind: JournalLinkKind) {
  return invokeCommand<JournalLink>('journal_link_create', { fromId, toId, kind });
}

/** 删除手记流转 */
export function journalLinkDelete(id: string) {
  return invokeCommand<void>('journal_link_delete', { id });
}

/** 列出一条手记引用的书摘 */
export function journalCitationList(entryId: string) {
  return invokeCommand<JournalCitation[]>('journal_citation_list', { entryId });
}

/** 引用一条书摘或读书笔记 */
export function journalCitationCreate(entryId: string, bookNoteId: string) {
  return invokeCommand<JournalCitation>('journal_citation_create', { entryId, bookNoteId });
}

/** 取消书摘引用 */
export function journalCitationDelete(id: string) {
  return invokeCommand<void>('journal_citation_delete', { id });
}
