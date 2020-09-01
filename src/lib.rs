// #![allow(warnings)]

use core::ptr::NonNull;
use std::fmt;

/// A simple smart pointer structure which uses to hold a large data set on the 
/// heap, and the total size of this structure should be just the size of the 
/// raw pointer:
///
/// - 8 bytes in 64 bit machine
/// - 4 bytes in 32 bit machine
///
/// We override the default `Deref` trait to just getting back the heap value reference
/// rather the `BlackBox` instance itself.
///
/// As we want to hold a raw pointer in this structure, and `Option<NonNull<T>>`
/// describes either a **valid pointer** or a **null pointer**.  That's why we use
/// `Option` here which only got 2 possible values:
///
/// - `Some()` - means **valid pointer**
/// - `None` - means **null pointer**
///
/// The **valid pointer** means:
///
/// 1. Non null, it must point to particular `<T>` instance.
/// 2. `<T>` instance should live on the **heap**.
pub struct BlackBox<T: ?Sized> {
    large_data_on_the_heap: Option<NonNull<T>>,
}

///
impl<T: fmt::Debug> BlackBox<T> {
    /// Creating instance, and the `large_data_set`'s ownership will be moved into
    /// the created instance.
    pub fn new(large_data_set: T) -> Self {
        // We box the original value here to MAKE SURE that value is allocated on the heap!!!
        let boxed_value = Box::new(large_data_set);

        // Convert `Box<T>` to `NonNull<T>` which is the raw pointer type
        let non_null = NonNull::from(Box::leak(boxed_value));

        BlackBox {
            large_data_on_the_heap: Some(non_null),
        }
    }
}

/// We want `{:?}` or `{:#?}` work for `BlackBox` instance, that's why we ask for
/// the `T` should implement the `fmt::Debug` trait
impl<T: fmt::Debug> fmt::Debug for BlackBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Just for printing the data (not move the ownership), that's why
        // return `&T` here rather the `T`. Wrap the result into `Option`
        // can deal with the no value case.
        let data_option_ref: Option<&T> = match self.large_data_on_the_heap {
            Some(data) => {
                // Get back raw pointer (point to) `T` from `NonNull<T>`, and keep
                // in mind that the `T` actually is a `Box<T>` here!!!
                let raw_ptr: *mut T = data.as_ptr();

                // `*raw_ptr` is the `Box<T>` value itself, but we want to return `&Box<T>`,
                // that's why we need the extra `&` here
                unsafe { Some(&*raw_ptr) }
            }
            None => None,
        };

        f.debug_struct("BlackBox")
            // As `Box<T>` implements the `fmt::Debug` trait, that's why the below
            // `field()` call will work.
            .field("large_data_on_the_heap", &data_option_ref)
            .finish()
    }
}

/// Override the default `deref` trait to get back the heap value reference rather 
/// than the structure instance itself, make it looks more natural and transparent.
impl<T> std::ops::Deref for BlackBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        println!("[ dereference happens >>>>>>>>>>>>>>>>>>>>> ]\n");

        // Here, we return `self.large_data_on_the_heap` reference rather than
        // return `&self`. As that's a raw pointer to `Box<T>`, then we need to
        // `take it out`.

        // `self.large_data_on_the_heap.as_ref()` returns `Option<&NonNull<T>>`,
        // `unwrap()` that return back `&NonNull<T>`. And `T` actually is a `Box<T>`!!!
        let option_ref: &NonNull<T> = self.large_data_on_the_heap.as_ref().unwrap();

        let raw_pointer = option_ref.as_ptr();
        unsafe { &*raw_pointer }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn heap_allocated_string_box() {
        let string_box: BlackBox<String>;

        // This `BlackBox<T>` mem size should be only the raw pointer size which:
        // 8 bytes in 64 bit machine
        // 4 bytes in 32 bit machine
        println!(
            "BlackBox<String> struct size: {}\n",
            mem::size_of::<BlackBox<String>>()
        );

        {
            // Simulate the very large size data on the heap:
            // This string take 24 bytes (22 bytes data + 2 bytes meta data in `String` type)
            let large_data_string_value = "Very large string data".to_owned();

            // `large_data_string_value`'s ownership will be taken (moved) into the `string_box`.
            // It means ONLY copy the meta data of the `String` type (2 bytes), NOT the head-allocated
            // string content itself (22 bytes), so that's cheap copy:)
            string_box = BlackBox::new(large_data_string_value);

            // This will cause `dereference`, that's why will get back a `String` value!!!
            // As the `clone()` only needs to copy the raw pointer size, so that's a cheap copy as
            // well.
            let temp_value: String = string_box.clone();

            // Should be the same size with `BlackBox<T>` (only the raw pointer size)
            println!("string_box size: {}\n", mem::size_of_val(&string_box));
            println!("string_box: {:#?}\n", &string_box);

            println!("temp_value size: {}", mem::size_of_val(&temp_value));
            println!("temp_value: {}\n", &temp_value);
        }

        // `large_data_string_value` variable out of scope, will be dropped, but the string content
        // which allocated on the heap already `moved into` `string_box`, that's why `string_box.large_data_string_value`
        // still available, u still can print the `string_box` with the original string content.
        println!("string_box: {:#?}\n", &string_box);

        // Cheap copy and dereference happens again
        let temp_value: String = string_box.clone();
        println!("temp_value: {}\n", &temp_value);
    }

    #[test]
    fn heap_allocated_struct_box() {
        #[derive(Debug, Clone)]
        struct Address {
            country: String,
            city: String,
            street: String,
        }

        #[derive(Debug, Clone)]
        struct Person {
            first_name: String,
            last_name: String,
            address: Address,
        }

        // As we need the struct instance allocated on the heap, so we use `Box` to wrap it.
        let person = Person {
            first_name: "Wison".to_owned(),
            last_name: "Ye".to_owned(),
            address: Address {
                country: "New Zealand".to_owned(),
                city: "Amazing City".to_owned(),
                street: "Wonderful Street".to_owned()
            },
        };

        // Should be 120 bytes
        println!("person size: {} bytes\n", mem::size_of_val(&person));
        println!("person: {:#?}", &person);

        let struct_box: BlackBox<Person> = BlackBox::new(person);

        // It should cause dereference `BlackBox` instance and get back the `Person` instance
        let temp_person_struct_value: Person = struct_box.clone();

        // Should be the same size with `BlackBox<T>` (only the raw pointer size)
        println!("struct_box size: {} bytes\n", mem::size_of_val(&struct_box));
        println!("struct_box: {:#?}\n", &struct_box);

        println!("temp_person_struct_value: {:#?}\n", &temp_person_struct_value);
        println!(
            "temp_person_struct_value size: {} bytes",
            mem::size_of_val(&temp_person_struct_value)
        );
    }
}
