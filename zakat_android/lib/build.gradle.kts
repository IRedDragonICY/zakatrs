plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("org.mozilla.rust-android-gradle.rust-android")
}

android {
    namespace = "com.github.ireddragonicy.zakatrs"
    compileSdk = 33

    defaultConfig {
        minSdk = 21
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
}

cargo {
    module = "../../" // Path to Cargo.toml directory (root)
    libname = "zakat" // Name of the library in Cargo.toml
    targets = listOf("arm", "arm64", "x86", "x86_64")
    apiLevel = 21
}

// Custom task to generate Kotlin bindings via UniFFI
// This assumes 'uniffi-bindgen' is installed in the system/CI 
// or run via cargo run.
tasks.register<Exec>("generateUniFFIBindings") {
    workingDir(project.projectDir.parentFile.parentFile) // Root of repo
    commandLine("cargo", "run", "--features=uniffi/cli", "--bin", "uniffi-bindgen", "generate", "--library", "target/release/libzakat.so", "--language", "kotlin", "--out-dir", "${project.buildDir}/generated/source/uniffi")
    
    // Dependent on cargo build being done first? 
    // For simplicity in this plan, we might need a more robust task dependency chain.
    // Use the uniffi-bindgen-googleye or similar if possible.
}

dependencies {
    implementation("net.jna:jna:5.13.0")
}
