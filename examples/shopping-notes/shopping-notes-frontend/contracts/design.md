# Frontend design reference

Mockups that pin each frontend spec's visual design: static HTML pages opened in a browser with no build. A mockup loads `mockups/mockup.js`, which links the stylesheet `mockups/mockup.css`, a snapshot of the approved design that `src/style.css` must match, and adds an "All mockups" link back to the index [mockups.html](mockups.html). Each state of a page sits in its own `data-state` block.

`e2e/design.spec.ts` holds each page to every state of its mockup: `toMatchAriaSnapshot` to its structure, `toHaveScreenshot` to its look, both against the mockup rendered in the same run, so no image is stored. To approve a design change, edit the mockup, then change `src/style.css` and the page until `npm run test:e2e` passes.

<!-- @spec contract fe-not-found-page -->

## Not found page

[mockups/pages/not-found.html](mockups/pages/not-found.html), state `default`

Not found page shown for an unmatched route: a "404" heading, a "Page not found" title, an explanatory line, and a "Back to main page" link, centered on the page.

<!-- @spec contract fe-add-shopping-item -->

## Add a shopping item

[mockups/pages/shopping-list.html](mockups/pages/shopping-list.html), states `empty` and `with-items`

"Add an item" text input with an "Add" button, at the top of the shopping list page under its title.

<!-- @spec contract fe-view-shopping-list -->

## View the shopping list

[mockups/pages/shopping-list.html](mockups/pages/shopping-list.html), states `with-items` and `empty`

`with-items`: one row per item showing its bought status and name. `empty`: a "No items yet" panel in place of the rows.

<!-- @spec contract fe-mark-item-bought -->

## Mark an item as bought

[mockups/pages/shopping-list.html](mockups/pages/shopping-list.html), state `with-items`

Item row with a checkbox named after its item ("Mark Milk as bought"); a bought item shows a checked box and a grey, struck-through name (the "Eggs" row).

<!-- @spec contract fe-delete-shopping-item -->

## Delete a shopping item

[mockups/pages/shopping-list.html](mockups/pages/shopping-list.html), state `with-items`

Item row with a red "Delete" action on the right.

<!-- @spec contract fe-clear-shopping-list -->

## Clear the shopping list

[mockups/pages/shopping-list.html](mockups/pages/shopping-list.html), state `with-items`

"Clear all" button below the item rows, shown only while the list has items.
