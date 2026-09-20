// @spec impl active-full-coverage#shows-subtotal, active-full-coverage#shows-tax
fn cart_subtotal_with_tax() {}

// @spec test active-full-coverage#shows-subtotal, active-full-coverage#shows-tax
fn cart_subtotal_with_tax_test() {}

// @spec impl in-progress-mixed#sends-email
fn send_shipping_email() {}

// @spec test in-progress-mixed#sends-email
fn send_shipping_email_test() {}

// @spec impl none-full-coverage#marks-item-tracked, none-full-coverage#notifies-on-price-drop
fn track_item_price() {}

// @spec test none-full-coverage#marks-item-tracked, none-full-coverage#notifies-on-price-drop
fn track_item_price_test() {}

// @spec impl none-with-gap#pauses-favorite
fn pause_favorite() {}

// @spec test none-with-gap#pauses-favorite
fn pause_favorite_test() {}

// @spec impl foreach-full-coverage#retries-with-new-card, foreach-full-coverage#retries-once-only
fn retry_payment() {}

// @spec test foreach-full-coverage#retries-with-new-card, foreach-full-coverage#retries-once-only
fn retry_payment_test() {}

// @spec impl foreach-with-gap#splits-across-cards
fn split_payment() {}
