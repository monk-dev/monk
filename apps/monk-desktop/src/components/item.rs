use dioxus::prelude::*;
use monk::types::Item;

#[derive(Props)]
pub struct ItemProps<'i> {
    item: &'i Item,
}

pub fn Item<'i>(cx: Scope<'i, ItemProps<'i>>) -> Element {
    rsx!(cx,
        div {
            class: "",
            p { "{cx.props.item.id}"}
        }
    )
}

// <div class="container w-1/2 mx-auto mt-10">
//   <article class="max-w-lg p-1 border rounded-md bg-gradient-to-r from-gray-50 to-gray-100">
//     <div class="flex flex-col">
//       <div class="flex flex-row justify-between items-center border-b">
//         <h1 class="font-serif font-bold text-lg">As We May Think</h1>
//         <div>
//           <a href="https://www.theatlantic.com/magazine/archive/1945/07/as-we-may-think/303881/" target="_blank">
//           <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
//             <path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" />
//             <path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" />
//           </svg>
//           </a>
//         </div>
//       </div>
//       <section class="text-sm py-1">
//         essentially monk but in the 1940's
//       </section>
//       <div class="flex flex-row justify-between items-center">
//         <small>2/15/2022</small>
//         <small class="font-mono">57bbf229-9d84-4b4f-9325-df6bf56dcfd4</small>
//       </div>
//     </div>
//   </article>
// <!-- Item {
//   id: a4c5972d-5e23-4031-85f5-4746a20818e8,
//   name: Some("As We May Think"),
//   url: Some("https://www.theatlantic.com/magazine/archive/1945/07/as-we-may-think/303881/"),
//   comment: Some("essentially monk but in the 1940's"),
//   tags: [], blob:
//     Some(Blob {
//       id: 566f15b8-2280-495b-aa52-80529c9791e6,
//       uri: "https://www.theatlantic.com/magazine/archive/1945/07/as-we-may-think/303881/",
//       hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
//       content_type: "text/html",
//       path: "downloads/a4c5972d-5e23-4031-85f5-4746a20818e8.html",
//       managed: true,
//       created_at: 2022-02-04T01:30:15.976427367Z }),
//   created_at: 2022-02-04T01:30:14.756194596Z } -->
// </div>
