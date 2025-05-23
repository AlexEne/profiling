#![allow(dead_code)]

//
// Example of marking up all functions on an impl block
//
struct Foo;

#[profiling::all_functions]
impl Foo {
    pub fn function1() {
        some_other_function(5);
    }

    #[profiling::skip]
    pub fn function2() {
        some_other_function(5);
    }
}

//
// Examples of marking up a single function
//

// This `profiling::function` attribute is equivalent to profiling::scope!(function_name)
#[profiling::function]
fn some_function() {
    burn_time(5);
}

#[profiling::function]
fn some_inner_function(_iteration_index: usize) {
    burn_time(10);
}

fn function_scope_function() {
    profiling::function_scope!();
    burn_time(10);
}

fn function_scope_function_with_data(_iteration_index: usize) {
    profiling::function_scope!(_iteration_index.to_string().as_str());
    burn_time(5);
}

//
// Example of multiple scopes in a single function
//
fn some_other_function(iterations: usize) {
    profiling::scope!("some_other_function");
    burn_time(5);

    {
        profiling::scope!("do iterations");
        for i in 0..iterations {
            profiling::scope!(
                "some_inner_function_that_sleeps",
                format!("other data {}", i).as_str()
            );

            // Mixing general profiling API calls with profiler-specific API calls is allowed
            #[cfg(feature = "profile-with-optick")]
            profiling::optick::tag!("extra_data", "MORE DATA");

            some_inner_function(i);
            burn_time(1);
        }
    }
}

// This function just spin-waits for some amount of time
fn burn_time(millis: u128) {
    let start_time = std::time::Instant::now();
    loop {
        if (std::time::Instant::now() - start_time).as_millis() > millis {
            break;
        }
    }
}

#[cfg(not(any(
    feature = "profile-with-optick",
    feature = "profile-with-puffin",
    feature = "profile-with-superluminal",
    feature = "profile-with-tracing",
    feature = "profile-with-tracy",
)))]
fn main() {
    println!("==================================================================================================");
    println!("No profiler feature flags were enabled. Since this is an example, this is probably a mistake.");
    println!("Please compile with a feature enabled to run this example.");
    println!("");
    println!("Example:");
    println!("    cargo run --example simple --features=\"profile-with-tracy\"");
    println!("");
    println!("Supported feature flags are documented here: https://github.com/aclysma/profiling#feature-flags");
    println!("");
    println!("Alternatively, try the demo-puffin example:");
    println!("    cd demo-puffin");
    println!("    cargo run --package demo-puffin");
    println!("==================================================================================================");
}

// Just check that one of these features was enabled because otherwise, nothing interesting will happen
#[cfg(any(
    feature = "profile-with-optick",
    feature = "profile-with-puffin",
    feature = "profile-with-superluminal",
    feature = "profile-with-tracing",
    feature = "profile-with-tracy",
))]
fn main() {
    #[cfg(feature = "profile-with-optick")]
    println!("optick");
    #[cfg(feature = "profile-with-puffin")]
    println!("puffin");
    #[cfg(feature = "profile-with-superluminal")]
    println!("superluminal");
    #[cfg(feature = "profile-with-tracing")]
    println!("tracing");
    #[cfg(feature = "profile-with-tracy")]
    println!("tracy");

    // Starting the Tracy client is necessary before any invoking any of its APIs
    #[cfg(feature = "profile-with-tracy")]
    tracy_client::Client::start();

    // Good to call this on any threads that are created to get clearer profiling results
    profiling::register_thread!("Main Thread");

    // Set up the tracy layer in the tracing crate. This is just an example using tracing. This is
    // not necessary if using tracy directly. (profile-with-tracy)
    #[cfg(feature = "profile-with-tracing")]
    {
        use tracing_subscriber::layer::SubscriberExt;
        tracing::subscriber::set_global_default(
            tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
        )
        .unwrap();
    }

    // Turn on tracing for puffin (you would still need to render/save this somehow!)
    #[cfg(feature = "profile-with-puffin")]
    profiling::puffin::set_scopes_on(true);

    println!("Starting loop, profiler can now be attached");

    // Test that using this macro multiple times in the same scope level will compile.
    //
    // optick backend currently won't work with multiple `profiling::scope!` in the same scope
    #[cfg(not(any(feature = "profile-with-optick")))]
    {
        profiling::scope!("Outer scope");
        burn_time(5);
        profiling::scope!("Inner scope");
        burn_time(5);
    }

    // Test that non-literals can be used
    //
    // Does not work with these three backends:
    #[cfg(not(any(
        feature = "profile-with-puffin",
        feature = "profile-with-tracing",
        feature = "profile-with-superluminal"
    )))]
    // optick backend currently won't work with multiple `profiling::scope!` in the same scope
    #[cfg(not(any(feature = "profile-with-optick")))]
    {
        let scope_name = String::from("Some scope name");
        profiling::scope!(&scope_name);
        burn_time(5);

        let another_scope_name = String::from("Another scope name");
        let some_data = String::from("Some data");
        profiling::scope!(&another_scope_name, &some_data);
        burn_time(5);
    }

    loop {
        // Generate some profiling info
        profiling::scope!("Main Thread");
        some_function();
        some_other_function(10);

        Foo::function1();
        Foo::function2();

        for i in 0..10 {
            function_scope_function();
            function_scope_function_with_data(i);
            burn_time(1);
        }

        println!("frame complete");

        // Finish the frame.
        profiling::finish_frame!();
    }
}
