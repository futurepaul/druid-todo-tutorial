Hello! My name is Paul and today I'd like to help you make a slightly-more-complex-than-hello-world app using [Druid, a GUI framework written in Rust][druid]. In the classic GUI tradition we'll be working on a simple todo app. You'll need some familiarity with Rust to follow along, especially Rust's concept of [Traits][traits], but I'll try not to assume too much familiarity because a lot of what I know about Rust has been learned in parallel with learning Druid and contributing to the project.

## 1. Setup

To get started let's create a new project: `cargo new druid-todo-tutorial`. Now `cd` into that folder and add `druid` as a dependency to the `Cargo.toml` file. This tutorial was written against Druid version 0.7: 

```toml
[dependencies]
druid = { version = "0.7", features = ["im"]}
```

The `"im"` feature flag is optional but it allows Druid to use immutable types from the `im-rs` project which ends up being an ergonomic win when setting up our app's state (we'll be storing our todos in an immutable `Vector`).

Druid uses native platform dependencies on Windows and Mac, but if you're on Linux you'll also need to [make sure you have GTK3 on your system][gtk3].

## 2. Hello world

Alright let's get a basic window on the screen. To save ourselves the hassle of refactoring later I'm going to split this code into three files right from the start:

### data.rs

This is where our application state, along with its relevant methods, will live. For now we'll just use an empty struct. State that we hand off to Druid must impl `Data`, which can be derived for structs containing many of the basic Rust types, as long as they're cheap to compare and cheap to clone. Druid uses cheap equality checks on the app state to know when it should re-render. 

```rust
use druid::Data;

#[derive(Clone, Data)]
pub struct AppState {}
```

### view.rs

Here is where we'll compose widgets in order to represent our UI. I'm creating a `build_ui` function that returns Druid's built-in `Label` widget with the static text of `"Hello"`. The function signature is `impl Widget<AppState>` but it would be also correct to say it simply returns a `Label<AppState>`. I use the `impl` style because once we start wrapping the label in various layout widgets the specific type we're returning will change, but they all impl Druid's `Widget` trait and that's all I care about.

```rust
use druid::{widget::Label, Widget};

use crate::data::*;

pub fn build_ui() -> impl Widget<AppState> {
    Label::new("Hello")
}
```

### main.rs

Druid uses `druid-shell` under the hood to work with the native platform on stuff like windows, drawing, and the event loop. Here we describe a `main_window` with a root widget of `build_ui` (which we just defined in `view.rs`), and an `initial_state`. Then we hand the window and state to `AppLauncher` which will now be in charge of drawing our app based on our widget tree and updating our state appropriately.

```rust
use druid::{AppLauncher, WindowDesc};

mod data;
use data::AppState;

mod view;
use view::build_ui;

pub fn main() {
    let main_window = WindowDesc::new(build_ui)
        .title("Todo Tutorial")
        .window_size((400.0, 400.0));

    let initial_state = AppState {};

    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}
```

When you run this with `cargo run` you should get a nice little window with the word "Hello" in the top left corner.

## 3. Creating a list

A todo list app needs a list of todos, so let's add that to our `data.rs`. We'll create a `TodoItem` struct and add a `im::Vector` of those to the `AppState`. I'm also deriving `Lens` for both of our structs, which I'll explain in a second. I'll also impl some `new` functions to make it easier to stub in dummy data. 

_If you you don't want to use an immutable `Vector` for the todo list, you can also use a traditional Rust `Vec`, however you can't derive `Data` for `Vec` automatically (remember `Data` needs to be cheap to compare and cheap to clone). To solve this, wrap the `Vec` in an `Arc` and you'll be good. `Vector` is easy to mutate without cloning, but this is also usually possible with `Arc` using `Arc::make_mut`_.

### data.rs

```rust
use druid::{im::Vector, Data, Lens};

#[derive(Clone, Data, Lens)]
pub struct AppState {
    todos: Vector<TodoItem>,
}

impl AppState {
    pub fn new(todos: Vec<TodoItem>) -> Self {
        Self {
            todos: Vector::from(todos),
        }
    }
}

#[derive(Clone, Data, Lens)]
pub struct TodoItem {
    done: bool,
    text: String,
}

impl TodoItem {
    pub fn new(text: &str) -> Self {
        Self {
            done: false,
            text: text.into(),
        }
    }
}
```

Now in our view we'll compose a widget to represent a single `TodoItem`. Then we'll change our `build_ui` function to return a list composed of these `todo_item` widgets.

### view.rs

```rust
use druid::{
    widget::{Checkbox, Flex, Label, List},
    Widget, WidgetExt,
};

use crate::data::*;

fn todo_item() -> impl Widget<TodoItem> {
    let checkbox = Checkbox::new("").lens(TodoItem::done);
    let label = Label::raw().lens(TodoItem::text);

    Flex::row().with_child(checkbox).with_flex_child(label, 1.)
}

pub fn build_ui() -> impl Widget<AppState> {
    List::new(todo_item).lens(AppState::todos)
}
```

Most of this should be pretty straightforward. We create a checkbox and label widget from Druid's standard toolkit, then put them inside a `Flex` row (layout in Druid is usually done in the flexbox style, similar to `Flutter` and to a lesser extent the web's flexbox, but you use an explicit `Flex` widget to do it). Then we pass the `todo_item` function to `List` which will use it to build each of its children.

What's interesting here is that if you look at `Druid's` implementation of `Checkbox`, it impls the `Widget` trait for `Widget<bool>`, meaning it can represent app state of type bool. But we're trying to display a `TodoItem` that contains a bool!

Meanwhile, `Label::raw()` constructs a `RawLabel` widget which is generic on `T: TextStorage` which we obviously haven't implemented for `TodoItem`.

Enter lenses. A lens is a datatype that gives access to a part of a larger data structure. Because we have derived `Lens` for our `TodoItem` struct, we can "lens" into the members of `TodoItem` to give these widgets only the portion of data they know how to work with. `.lens(TodoItem::done)` gives `Checkbox` the `bool` it craves, while `.lens(TodoItem::text)` gives `RawLabel` a `String`, for which `Druid` has already implemented `TextStorage`. 

We don't need to do any lensing for the `Flex` widget because it doesn't need to look at its data, it simply passes it along to its children (this is common for many of the built-in layout widgets). The `List` widget requires a `Data` that impls its `ListIter` trait, but kindly offers default implmentations for a few basic collections, including `im::Vector`, so we just lens our `AppState` down to to `todos` and `List` knows what to do from there. 

_Lensing is one of Druid's hardest conceptual hurdles to climb, so don't stress if it doesn't click right away. The [Druid community][zulip] is always happy to help if you get stuck._

Now let's build our new app state:

### main.rs

```rust
    let todos = vec![TodoItem::new("thing one"), TodoItem::new("thing two")];
    let initial_state = AppState::new(todos);
```

And re-run the app. You should end up with a nice little two-item todo list.

## 4. Create more todos

Now let's make it so we can add todos at runtime using a textbox. We'll need somewhere to store this textbox's state, so we'll add it to the top-level AppState. We'll also add two more methods to `AppState` which I'll explain in a second:

### data.rs

```rust
#[derive(Clone, Data, Lens)]
pub struct AppState {
    new_todo: String,
    todos: Vector<TodoItem>,
}

impl AppState {
    pub fn new(todos: Vec<TodoItem>) -> Self {
        Self {
            new_todo: "".into(),
            todos: Vector::from(todos),
        }
    }

    fn add_todo(&mut self) {
        self.todos.push_front(TodoItem::new(&self.new_todo));
        self.new_todo = "".into();
    }

    pub fn click_add(_ctx: &mut EventCtx, data: &mut Self, _env: &Env) {
        data.add_todo();
    }
}
```

### view.rs

Now in our view we'll create a new function that impls `Widget<AppState>`:

```rust
fn new_todo_textbox() -> impl Widget<AppState> {
    let new_todo_textbox = TextBox::new()
        .with_placeholder("Add a new todo")
        .expand_width()
        .lens(AppState::new_todo);

    let add_todo_button = Button::new("Add").on_click(AppState::click_add);

    Flex::row()
        .with_flex_child(new_todo_textbox, 1.)
        .with_child(add_todo_button)
}
```

And add it to our main ui:

```rust
pub fn build_ui() -> impl Widget<AppState> {
    Flex::column()
        .with_child(new_todo_textbox())
        .with_child(List::new(todo_item).lens(AppState::todos))
}
```

The important thing to understand here is the `.on_click(AppState::click_add)` on our `Button` widget. If I hover over `on_click` in VS Code with Rust Analyzer hooked up, I see this delightful function signature:

```rust
pub fn on_click(self, f: impl Fn(&mut EventCtx, &mut T, &Env) + 'static) -> ControllerHost<Self, Click<T>>
```

The `impl Fn(&mut EventCtx, &mut T, &Env) + 'static` function is satisfied by the `click_add` method we put on `AppState`. It's also totally fine to write this as a closure inline, but I find it's a little messy:

```rust
let add_todo_button = Button::new("Add")
        .on_click(|_ctx: &mut EventCtx, data: &mut AppState, _env: &Env| data.add_todo());
```

What this `on_click` method ultimately boils down to is an implementation of Druid's `Widget` trait with only the `event` portion defined by us. That is to say, we're creating a `Widget` to wrap our `Button` widget, and we're going to intercept click events and do something with them, but otherwise we'll just let button handle everything else (everything else including other events like mouse hover, and other parts of the `Widget` impl like `layout`, `update`, and `paint`). 

## 5. Saving our state to disk

A todo app is of limited utility if it doesn't persist its state, so let's do some serializing. This is mostly fairly standard Rust stuff so I'm not going to go into too much detail here.

First we'll add the `serde` dependencies:

### Cargo.toml

```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### data.rs

Now we derive serde's `Serialize` and `Deserialize` for `TodoItem`:

```rust
#[derive(Clone, Data, Lens, Serialize, Deserialize)]
pub struct TodoItem {
    done: bool,
    pub text: String,
}
```

Then we can add load and save methods to `AppState`. Note that we need to convert our `Vector` into a standard Rust `Vec` for the sake of serialization.

```rust
    pub fn save_to_json(&self) -> Result<(), Error> {
        let todo_vec: Vec<TodoItem> = self.todos.iter().map(|item| item.to_owned()).collect();
        let serialized = serde_json::to_string_pretty(&todo_vec)?;
        std::fs::write("todos.json", serialized)?;
        Ok(())
    }

    pub fn load_from_json() -> Self {
        let file = File::open("todos.json");

        match file {
            Ok(file) => {
                let reader = BufReader::new(file);
                let todos: Vec<TodoItem> = serde_json::from_reader(reader).unwrap_or(vec![]);
                Self {
                    todos: Vector::from(todos),
                    new_todo: String::new(),
                }
            }
            Err(_) => Self {
                todos: Vector::new(),
                new_todo: String::new(),
            },
        }
    }
```

Now we can update our `add_todo` method to include a call to `save_to_json`:

```rust
    fn add_todo(&mut self) {
        self.todos.push_front(TodoItem::new(&self.new_todo));
        self.new_todo = "".into();
        self.save_to_json().unwrap();
    }
```

### main.rs

And now we can now generate our `initial_state` from the `.json` file during setup (this will default to an empty state if there is none):

```rust
let initial_state = AppState::load_from_json();
```

Now if you run this you should get an empty todo list. If you add a couple items, then close and reopen the app, they should be persisted! With our hardcoded `"todos.json"` path this file will be generated in the root folder of our project, though obviously you can use any path you'd like.  

## 6. Saving the "done" state

You might've noticed a glaring flaw in our serialization plan: we're only saving to disk when we add a new todo. We don't have any way right now to react to when a todo is marked or unmarked as "done".

This highlights an interesting constraint when building Druid apps: it's very easy to "lens down" to smaller and smaller portions of the AppState. But once a widget deep in the tree (in this case, `Checkbox<bool>`) needs to act on a different portion of the tree or call a method on the root `AppState`, we need an escape hatch. There are a few ways to go about this. None of them are perfectly elegant, but for most of my own encounters with this situation I've been using what I heretically call the "Elm style" of firing a `Command` (a special kind of event that's internal to Druid) from the leaf widget that will be handled by the root of the app.

In practice we'll be using the same machinery that powers the `on_click` feature we saw earlier: a `Controller`.

Let's create two new files:

### delegate.rs

```rust
use druid::{AppDelegate, Command, DelegateCtx, Env, Handled, Selector, Target};

use crate::data::AppState;

pub const SAVE: Selector = Selector::new("todo.save");

pub struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        if cmd.is(SAVE) {
            data.save_to_json().unwrap();
            Handled::Yes
        } else {
            println!("cmd forwarded: {:?}", cmd);
            Handled::No
        }
    }
}
```

The `AppDelegate` will wrap our whole app and intercept and handle the `Commands` we dispatch. Any widget can handle a `Command`, but this offers a nice and tidy place to handle top-level stuff and call methods on our `AppState`.

A specific `Command` is identified by its `Selector`, which we define here with the const `SAVE`. Then in the `AppDelegate` we match on `cmd.is(SAVE)`. If it is save, we call our `save_to_json` function and declare that we've `Handled` the `Command`. For all other commands we'll say `Handled::No` so Druid knows to propogate that `Command` down the tree.

### controllers.rs

```rust
use druid::{widget::Controller, Env, UpdateCtx, Widget};

use crate::data::*;
use crate::delegate::SAVE;

pub struct TodoItemController;

impl<W: Widget<TodoItem>> Controller<TodoItem, W> for TodoItemController {
    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &TodoItem,
        data: &TodoItem,
        env: &Env,
    ) {
        if old_data.done != data.done {
            ctx.submit_command(SAVE);
        }
        child.update(ctx, old_data, data, env);
    }
}
```

This `Controller` is what will wrap our `TodoItem` widget. You'll see it's generic on `W: Widget<TodoItem>`, so any widget that satisfies `impl Widget<TodoItem>` is fair game. Unlike the `on_click` handler, which was overriding the `event` method of `Widget`, this `Controller` is overriding the `update` method. It's sitting in the widget tree and examining incoming changes to `data`. It checks if `old_data.done` is different than `data.done`, and if so it submits a command to the `ctx` which be sent to the top of the tree to be handled by our `AppDelegate`.   

Now let's wire these up to our app.

### main.rs

Make sure to declare the new modules:

```rust
mod controllers;
mod delegate;
use delegate::Delegate;
```

And then call the `AppLauncher` with the `Delegate`:

```rust
    AppLauncher::with_window(main_window)
        .delegate(Delegate {})
        .launch(initial_state)
        .expect("Failed to launch application");
```

### view.rs

Append the `TodoItemController` to `todo_item`'s return statement:

```rust
use crate::controllers::TodoItemController;

...

fn todo_item() -> impl Widget<TodoItem> {
    let checkbox = Checkbox::new("").lens(TodoItem::done);
    let label = Label::raw().lens(TodoItem::text);

    Flex::row()
        .with_child(checkbox)
        .with_flex_child(label, 1.)
        .controller(TodoItemController)
}
```

Now when you run the app and toggle todos that state should be saved to `todos.json`.

## 7. Deleting a todo

I was hoping you wouldn't notice that all of our todos are currently permanent. Very well, let's delete some!

The quick and easy way is to add a "Clear completed" method to our `AppState`. Let's do that first:

### data.rs

```rust
    pub fn clear_completed(_ctx: &mut EventCtx, data: &mut Self, _env: &Env) {
        data.todos.retain(|item| !item.done);

        data.save_to_json().unwrap();
    }
```

Remember the arguments like `ctx` and `env` are because we'll be using this with an `on_click` handler.

### view.rs

```rust
pub fn build_ui() -> impl Widget<AppState> {
    let clear_completed_button = Button::new("Clear completed").on_click(AppState::clear_completed);

    Flex::column()
        .with_child(new_todo_textbox())
        .with_child(List::new(todo_item).lens(AppState::todos))
        .with_flex_spacer(1.)
        .with_child(clear_completed_button)
}
```

We pass the `AppState::clear_completed_button` method to this `Button`'s `on_click`, do a little bit of flex spacer-ing, and we're done!

Now we can run the app and clear the completed todos.

But let's tackle the harder case of deleting a single todo from the todo itself. Again, because a `TodoItem` is lensed down to a single element of a `Vector`, it doesn't make very much sense for it to delete "itself", so instead we'll want to ask the `AppDelegate` to delete us. In order to do this we'll need some sort of stable identity. It might be overkill but I enjoy using the `uuid` crate:

### Cargo.toml

```
uuid = { version = "0.8.1", features = ["serde", "v4"] }
```

Including the `serde` feature means we'll be able to serialize this id. It's really a beautiful ecosystem!

Okay let's update our `TodoItem` struct:

### data.rs

```rust
#[derive(Clone, Data, Lens, Serialize, Deserialize)]
pub struct TodoItem {
    #[data(same_fn = "PartialEq::eq")]
    pub id: Uuid,
    pub done: bool,
    pub text: String,
}

impl TodoItem {
    pub fn new(text: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            done: false,
            text: text.into(),
        }
    }
}
```

Because `Uuid` doesn't impl Druid's `Data` trait, we can manually specify that Druid should use `same_fn = "PartialEq::eq"` to derive `Data`, which is fast in this case because `Uuid` is 128 bits all on the stack. For truly exceptionally cases you can also of course impl `Data` manually, and of course you can always wrap your type in an `Arc`. Just remember the mantra: "cheap to compare and cheap to clone."

Now if you run the app you should see no todos in your list, even if you had some in `todos.json`. That's because we didn't make this backwards compatible, so serde failed to deserialize, and we just defaulted to an empty state. But if you create some more todos you should be seeing some uuids now.

Okay so now with the help of our `Uuid` let's wire up a "Delete" button on each `todo_item`. 

First let's add a method to `AppState` to do the actual deleting. This looks a lot like our `clear_completed` function:

```rust
    pub fn delete_todo(&mut self, id: &Uuid) {
        self.todos.retain(|item| &item.id != id);

        self.save_to_json().unwrap();
    }
```

### delegate.rs

We'll add a new `Selector`, but this one will take a "payload" of `Uuid`:

```rust
pub const DELETE: Selector<Uuid> = Selector::new("todo.delete");
```

And handle that `Command` in the `AppDelegate`, pulling out the payload value with `cmd.get`:

```rust
        if cmd.is(SAVE) {
            data.save_to_json().unwrap();
            Handled::Yes
        } else if let Some(id) = cmd.get(DELETE) {
            data.delete_todo(id);
            Handled::Yes
        } else {
            println!("cmd forwarded: {:?}", cmd);
            Handled::No
        }
```

### data.rs

Back in data we'll add a `click_delete` method to `TodoItem`:

```rust
    pub fn click_delete(ctx: &mut EventCtx, data: &mut Self, _env: &Env) {
        ctx.submit_command(DELETE.with(data.id));
    }
```

### view.rs

Finally, we'll create the "Delete" button and add it to our `todo_item`'s `Flex` row.

```rust
    let delete_button = Button::new("Delete").on_click(TodoItem::click_delete);

    Flex::row()
        .with_child(checkbox)
        .with_child(label)
        .with_flex_spacer(1.)
        .with_child(delete_button)
        .controller(TodoItemController
    let delete_button = Button::new("Delete").on_click(TodoItem::click_delete);
```

Now if you re-run the app you'll have full delete functionality!

## 8. What's next?

In preparing for this tutorial I made a [fuller-featured version of this app][druid-todo], including styling and editable todos. Hopefully the Druid mechanisms I've shown you here should give you enough context to read that code and figure out what's going on. When Druid is more mature I'd like to revisit this tutorial with new best practices and do some good styling to make our app look really polished.

Thanks for reading!

[druid]: https://github.com/linebender/druid
[traits]: https://doc.rust-lang.org/book/ch10-02-traits.html
[gtk3]: https://github.com/linebender/druid#using-druid
[zulip]: https://xi.zulipchat.com/
[druid-todo]: https://github.com/futurepaul/druid-todo
