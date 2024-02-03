### `on conflict`

`excluded.` means "get new value"

```sql
insert into {table}
  (id, column, ..)
values (..)
on conflict(id)
  do update set column=excluded.column
```
