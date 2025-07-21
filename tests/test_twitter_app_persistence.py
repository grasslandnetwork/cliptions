#!/usr/bin/env python3
"""
Integration test for Twitter data fetcher login persistence.
This test uses the ACTUAL twitter_data_fetcher.py application to verify 
that Twitter login sessions persist between runs.
"""

import asyncio
import os
import pathlib
import json
import subprocess
import sys
import time

def test_twitter_app_persistence():
    """Test that the actual twitter_data_fetcher.py maintains login persistence"""
    
    print("🐦 Testing ACTUAL Twitter Data Fetcher Login Persistence")
    print("=" * 60)
    
    # Check for Twitter credentials
    twitter_username = os.environ.get('TWITTER_NAME')
    twitter_password = os.environ.get('TWITTER_PASSWORD')
    
    if not twitter_username or not twitter_password:
        print("❌ ERROR: TWITTER_NAME and TWITTER_PASSWORD environment variables must be set")
        print("Set them using:")
        print("  export TWITTER_NAME='your_twitter_username'")
        print("  export TWITTER_PASSWORD='your_twitter_password'")
        return False
    
    print(f"✅ Twitter credentials found for user: {twitter_username}")
    
    # Get paths
    script_dir = pathlib.Path(__file__).parent.parent  # Go up to project root
    twitter_fetcher = script_dir / "browser-use" / "twitter_data_fetcher.py"
    browser_data_dir = script_dir / "browser-use" / "browser_data"
    cookies_file = browser_data_dir / "twitter_cookies.json"
    
    print(f"Testing application: {twitter_fetcher}")
    print(f"Browser data directory: {browser_data_dir}")
    print(f"Cookies file: {cookies_file}")
    
    if not twitter_fetcher.exists():
        print(f"❌ ERROR: Twitter data fetcher not found at {twitter_fetcher}")
        return False
    
    # Remove existing cookies to start fresh
    if cookies_file.exists():
        cookies_file.unlink()
        print("🧹 Removed existing Twitter cookies file to start fresh")
    
    try:
        print("\n" + "=" * 60)
        print("🚀 RUN 1: First run - should login to Twitter")
        print("=" * 60)
        
        # Run 1: First execution (should login)
        start_time = time.time()
        result1 = subprocess.run([
            sys.executable, str(twitter_fetcher),
            "--round", "0",
            "--target-time", "20250223_133057EST",
            "--debug"
        ], 
        cwd=str(script_dir),
        capture_output=True, 
        text=True,
        timeout=120  # 2 minute timeout
        )
        
        run1_duration = time.time() - start_time
        
        print(f"Run 1 completed in {run1_duration:.1f} seconds")
        print(f"Exit code: {result1.returncode}")
        
        if result1.stdout:
            print("STDOUT:")
            print(result1.stdout[-1000:])  # Last 1000 chars to avoid spam
        
        if result1.stderr:
            print("STDERR:")
            print(result1.stderr[-500:])  # Last 500 chars
        
        # Check if cookies file was created
        cookies_exist_after_run1 = cookies_file.exists()
        cookies_size_after_run1 = cookies_file.stat().st_size if cookies_exist_after_run1 else 0
        
        print(f"\nAfter Run 1:")
        print(f"  Cookies file exists: {cookies_exist_after_run1}")
        print(f"  Cookies file size: {cookies_size_after_run1} bytes")
        
        if cookies_exist_after_run1 and cookies_size_after_run1 > 2:
            print("✅ Run 1: Cookies file created and populated!")
            
            # Parse cookies to see what we got
            try:
                with open(cookies_file, 'r') as f:
                    cookies_data = json.load(f)
                print(f"  Contains {len(cookies_data)} cookies")
                
                # Look for Twitter-specific cookies
                twitter_cookies = [c for c in cookies_data if isinstance(c, dict) and 'name' in c and 
                                 any(keyword in c.get('name', '').lower() for keyword in 
                                     ['twitter', 'auth', 'session', 'csrf', 'twid', 'ct0'])]
                print(f"  Found {len(twitter_cookies)} Twitter-related cookies")
                
            except Exception as e:
                print(f"  Could not parse cookies: {e}")
        else:
            print("⚠️ Run 1: No cookies file created or file is empty")
        
        # Wait a moment between runs
        print("\n⏳ Waiting 3 seconds between runs...")
        time.sleep(3)
        
        print("\n" + "=" * 60)
        print("🔄 RUN 2: Second run - should reuse existing login")
        print("=" * 60)
        
        # Run 2: Second execution (should reuse login)
        start_time = time.time()
        result2 = subprocess.run([
            sys.executable, str(twitter_fetcher),
            "--round", "0", 
            "--target-time", "20250223_133057EST",
            "--debug"
        ],
        cwd=str(script_dir),
        capture_output=True,
        text=True,
        timeout=120  # 2 minute timeout
        )
        
        run2_duration = time.time() - start_time
        
        print(f"Run 2 completed in {run2_duration:.1f} seconds")
        print(f"Exit code: {result2.returncode}")
        
        if result2.stdout:
            print("STDOUT:")
            print(result2.stdout[-1000:])  # Last 1000 chars
        
        if result2.stderr:
            print("STDERR:")
            print(result2.stderr[-500:])  # Last 500 chars
        
        # Check cookies file after second run
        cookies_exist_after_run2 = cookies_file.exists()
        cookies_size_after_run2 = cookies_file.stat().st_size if cookies_exist_after_run2 else 0
        
        print(f"\nAfter Run 2:")
        print(f"  Cookies file exists: {cookies_exist_after_run2}")
        print(f"  Cookies file size: {cookies_size_after_run2} bytes")
        
        # Analyze results
        print("\n" + "=" * 60)
        print("📊 ANALYSIS")
        print("=" * 60)
        
        run1_success = result1.returncode == 0
        run2_success = result2.returncode == 0
        cookies_created = cookies_exist_after_run1 and cookies_size_after_run1 > 2
        cookies_persisted = cookies_exist_after_run2 and cookies_size_after_run2 > 2
        
        print(f"Run 1 successful: {run1_success}")
        print(f"Run 2 successful: {run2_success}")
        print(f"Cookies created: {cookies_created}")
        print(f"Cookies persisted: {cookies_persisted}")
        print(f"Run 1 duration: {run1_duration:.1f}s")
        print(f"Run 2 duration: {run2_duration:.1f}s")
        
        # Check for login-related messages in output
        run1_output = (result1.stdout + result1.stderr).lower()
        run2_output = (result2.stdout + result2.stderr).lower()
        
        login_keywords = ["login", "password", "username", "sign in", "authenticate"]
        run1_has_login = any(keyword in run1_output for keyword in login_keywords)
        run2_has_login = any(keyword in run2_output for keyword in login_keywords)
        
        print(f"Run 1 mentions login: {run1_has_login}")
        print(f"Run 2 mentions login: {run2_has_login}")
        
        # Determine overall success
        persistence_working = (
            run1_success and run2_success and 
            cookies_created and cookies_persisted and
            run1_has_login and not run2_has_login  # First run logs in, second doesn't need to
        )
        
        print(f"\n🎯 OVERALL RESULT: {'✅ PASSED' if persistence_working else '❌ FAILED'}")
        
        if persistence_working:
            print("✅ Twitter login persistence is working in the actual application!")
            print("✅ First run created login session")
            print("✅ Second run reused existing session")
            print("✅ Cookies are properly saved and loaded")
            if run2_duration < run1_duration * 0.8:
                print("✅ Second run was faster (likely due to reused session)")
        else:
            print("❌ Twitter login persistence test failed")
            if not run1_success:
                print("❌ First run failed")
            if not run2_success:
                print("❌ Second run failed")
            if not cookies_created:
                print("❌ No cookies were created after first run")
            if not cookies_persisted:
                print("❌ Cookies did not persist to second run")
        
        return persistence_working
        
    except subprocess.TimeoutExpired:
        print("❌ ERROR: Test timed out - application took too long to run")
        return False
    except Exception as e:
        print(f"❌ ERROR: Test failed with exception: {e}")
        return False
    
    finally:
        print(f"\n🔒 Twitter cookies preserved at: {cookies_file}")
        print("🔒 Keep the cookies file secure and don't share it.")

if __name__ == "__main__":
    success = test_twitter_app_persistence()
    sys.exit(0 if success else 1) 