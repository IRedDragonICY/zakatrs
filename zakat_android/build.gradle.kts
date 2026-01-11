// Top-level build file
plugins {
    id("com.android.library") version "8.1.0" apply false
    id("org.jetbrains.kotlin.android") version "2.3.0" apply false
    id("org.mozilla.rust-android-gradle.rust-android") version "0.9.6" apply false
}

buildscript {
    repositories {
        google()
        mavenCentral()
    }
}
