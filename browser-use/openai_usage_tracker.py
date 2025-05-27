#!/usr/bin/env python3
"""
OpenAI Usage and Cost Tracker

This module uses the OpenAI admin token to fetch actual usage and cost data
from the OpenAI Usage and Costs APIs, providing accurate tracking instead of estimates.
"""

import os
import json
import requests
import time
from datetime import datetime, timedelta, timezone
from pathlib import Path
import logging

log = logging.getLogger(__name__)

class OpenAIUsageTracker:
    """Track actual OpenAI usage and costs using the admin API"""
    
    def __init__(self):
        self.admin_api_key = os.getenv('OPENAI_API_KEY_FOR_USAGE_AND_COSTS')
        if not self.admin_api_key:
            raise ValueError("OPENAI_API_KEY_FOR_USAGE_AND_COSTS environment variable is required")
        
        self.headers = {
            "Authorization": f"Bearer {self.admin_api_key}",
            "Content-Type": "application/json",
        }
        
        self.usage_url = "https://api.openai.com/v1/organization/usage/completions"
        self.costs_url = "https://api.openai.com/v1/organization/costs"
        
        # Create data directory if it doesn't exist
        self.data_dir = Path('./browser_data')
        self.data_dir.mkdir(exist_ok=True)
        
        self.usage_file = self.data_dir / 'openai_actual_usage.json'
        self.costs_file = self.data_dir / 'openai_actual_costs.json'

    def get_paginated_data(self, url, params):
        """Fetch all data from a paginated API endpoint"""
        all_data = []
        page_cursor = None
        
        while True:
            if page_cursor:
                params["page"] = page_cursor
            
            try:
                response = requests.get(url, headers=self.headers, params=params)
                response.raise_for_status()
                
                data_json = response.json()
                all_data.extend(data_json.get("data", []))
                
                page_cursor = data_json.get("next_page")
                if not page_cursor:
                    break
                    
            except requests.exceptions.RequestException as e:
                log.error(f"Error fetching data from {url}: {e}")
                break
        
        return all_data

    def fetch_usage_data(self, days_back=7, project_ids=None):
        """Fetch usage data for the last N days, optionally filtered by project IDs"""
        start_time = int(time.time()) - (days_back * 24 * 60 * 60)
        
        params = {
            "start_time": start_time,
            "bucket_width": "1d",
            "group_by": ["model", "project_id"],
            "limit": days_back
        }
        
        # Add project filter if specified
        if project_ids:
            params["project_ids"] = project_ids
            log.info(f"Fetching usage data for last {days_back} days, filtered by projects: {project_ids}")
        else:
            log.info(f"Fetching usage data for last {days_back} days (all projects)")
        
        usage_data = self.get_paginated_data(self.usage_url, params)
        
        if usage_data:
            # Save raw usage data
            with open(self.usage_file, 'w') as f:
                json.dump({
                    'last_updated': datetime.now().isoformat(),
                    'days_back': days_back,
                    'data': usage_data
                }, f, indent=2)
            log.info(f"✅ Usage data saved to {self.usage_file}")
        
        return usage_data

    def fetch_costs_data(self, days_back=7, project_ids=None):
        """Fetch cost data for the last N days, optionally filtered by project IDs"""
        start_time = int(time.time()) - (days_back * 24 * 60 * 60)
        
        params = {
            "start_time": start_time,
            "bucket_width": "1d",
            "group_by": ["line_item", "project_id"],
            "limit": days_back
        }
        
        # Add project filter if specified
        if project_ids:
            params["project_ids"] = project_ids
            log.info(f"Fetching cost data for last {days_back} days, filtered by projects: {project_ids}")
        else:
            log.info(f"Fetching cost data for last {days_back} days (all projects)")
        
        costs_data = self.get_paginated_data(self.costs_url, params)
        
        if costs_data:
            # Save raw cost data
            with open(self.costs_file, 'w') as f:
                json.dump({
                    'last_updated': datetime.now().isoformat(),
                    'days_back': days_back,
                    'data': costs_data
                }, f, indent=2)
            log.info(f"✅ Cost data saved to {self.costs_file}")
        
        return costs_data

    def get_daily_costs(self, target_date=None):
        """Get actual costs for a specific date (timezone-aware)"""
        if target_date is None:
            target_date = datetime.now().date()
        elif isinstance(target_date, str):
            target_date = datetime.strptime(target_date, '%Y-%m-%d').date()
        
        # Load cost data
        if not self.costs_file.exists():
            log.warning("No cost data available. Run fetch_costs_data() first.")
            return None
        
        with open(self.costs_file, 'r') as f:
            cost_data = json.load(f)
        
        total_cost = 0
        daily_breakdown = {}
        
        for bucket in cost_data.get('data', []):
            # Use UTC dates directly - no timezone conversion needed
            bucket_start_utc = datetime.fromtimestamp(bucket['start_time'], tz=timezone.utc)
            bucket_date_utc = bucket_start_utc.date()
            
            # Only include costs if this bucket's UTC date matches the target date
            if bucket_date_utc == target_date:
                for result in bucket.get('results', []):
                    amount = result.get('amount', {}).get('value', 0)
                    line_item = result.get('line_item', 'unknown')
                    project_id = result.get('project_id', 'unknown')
                    
                    total_cost += amount
                    
                    key = f"{line_item} ({project_id})"
                    if key not in daily_breakdown:
                        daily_breakdown[key] = 0
                    daily_breakdown[key] += amount
        
        return {
            'date': target_date.isoformat(),
            'total_cost_usd': total_cost,
            'breakdown': daily_breakdown
        }

    def get_daily_usage(self, target_date=None):
        """Get actual usage for a specific date (timezone-aware)"""
        if target_date is None:
            target_date = datetime.now().date()
        elif isinstance(target_date, str):
            target_date = datetime.strptime(target_date, '%Y-%m-%d').date()
        
        # Load usage data
        if not self.usage_file.exists():
            log.warning("No usage data available. Run fetch_usage_data() first.")
            return None
        
        with open(self.usage_file, 'r') as f:
            usage_data = json.load(f)
        
        total_input_tokens = 0
        total_output_tokens = 0
        total_requests = 0
        model_breakdown = {}
        
        for bucket in usage_data.get('data', []):
            # Use UTC dates directly - no timezone conversion needed
            bucket_start_utc = datetime.fromtimestamp(bucket['start_time'], tz=timezone.utc)
            bucket_date_utc = bucket_start_utc.date()
            
            # Only include usage if this bucket's UTC date matches the target date
            if bucket_date_utc == target_date:
                for result in bucket.get('results', []):
                    input_tokens = result.get('input_tokens', 0)
                    output_tokens = result.get('output_tokens', 0)
                    requests = result.get('num_model_requests', 0)
                    model = result.get('model', 'unknown')
                    project_id = result.get('project_id', 'unknown')
                    
                    total_input_tokens += input_tokens
                    total_output_tokens += output_tokens
                    total_requests += requests
                    
                    key = f"{model} ({project_id})"
                    if key not in model_breakdown:
                        model_breakdown[key] = {
                            'input_tokens': 0,
                            'output_tokens': 0,
                            'requests': 0
                        }
                    
                    model_breakdown[key]['input_tokens'] += input_tokens
                    model_breakdown[key]['output_tokens'] += output_tokens
                    model_breakdown[key]['requests'] += requests
        
        return {
            'date': target_date.isoformat(),
            'total_input_tokens': total_input_tokens,
            'total_output_tokens': total_output_tokens,
            'total_requests': total_requests,
            'breakdown': model_breakdown
        }

    def sync_daily_data(self, target_date=None):
        """Sync both usage and cost data for a specific date"""
        if target_date is None:
            target_date = datetime.now().date()
        
        log.info(f"Syncing OpenAI data for {target_date}")
        
        # Fetch fresh data (last 7 days to ensure we have the target date)
        self.fetch_usage_data(days_back=7)
        self.fetch_costs_data(days_back=7)
        
        # Get the specific day's data
        usage = self.get_daily_usage(target_date)
        costs = self.get_daily_costs(target_date)
        
        return {
            'usage': usage,
            'costs': costs
        }

    def get_utc_daily_summary(self, days_back=7):
        """Get a UTC-aligned daily summary of costs and usage for the last N days"""
        # Load both cost and usage data
        if not self.costs_file.exists() or not self.usage_file.exists():
            log.warning("Cost or usage data not available. Run sync_daily_data() first.")
            return None
        
        with open(self.costs_file, 'r') as f:
            cost_data = json.load(f)
        
        with open(self.usage_file, 'r') as f:
            usage_data = json.load(f)
        
        # Get current timezone info
        local_tz = datetime.now().astimezone().tzinfo
        
        # Group data by local date
        daily_summary = {}
        
        # Process cost data
        for bucket in cost_data.get('data', []):
            bucket_start_utc = datetime.fromtimestamp(bucket['start_time'], tz=timezone.utc)
            bucket_date_utc = bucket_start_utc.date()
            date_str = bucket_date_utc.isoformat()
            
            if date_str not in daily_summary:
                daily_summary[date_str] = {
                    'date': date_str,
                    'total_cost': 0,
                    'total_input_tokens': 0,
                    'total_output_tokens': 0,
                    'total_requests': 0,
                    'timezone': 'UTC'
                }
            
            # Add costs for this UTC date
            for result in bucket.get('results', []):
                daily_summary[date_str]['total_cost'] += result.get('amount', {}).get('value', 0)
        
        # Process usage data
        for bucket in usage_data.get('data', []):
            bucket_start_utc = datetime.fromtimestamp(bucket['start_time'], tz=timezone.utc)
            bucket_date_utc = bucket_start_utc.date()
            date_str = bucket_date_utc.isoformat()
            
            if date_str not in daily_summary:
                daily_summary[date_str] = {
                    'date': date_str,
                    'total_cost': 0,
                    'total_input_tokens': 0,
                    'total_output_tokens': 0,
                    'total_requests': 0,
                    'timezone': 'UTC'
                }
            
            # Add usage for this UTC date
            for result in bucket.get('results', []):
                daily_summary[date_str]['total_input_tokens'] += result.get('input_tokens', 0)
                daily_summary[date_str]['total_output_tokens'] += result.get('output_tokens', 0)
                daily_summary[date_str]['total_requests'] += result.get('num_model_requests', 0)
        
        # Sort by date (most recent first)
        sorted_dates = sorted(daily_summary.keys(), reverse=True)
        return [daily_summary[date] for date in sorted_dates[:days_back]]

    def compare_with_estimates(self, target_date=None):
        """Compare actual costs with our estimated costs"""
        if target_date is None:
            target_date = datetime.now().date()
        
        # Get actual data
        actual_costs = self.get_daily_costs(target_date)
        actual_usage = self.get_daily_usage(target_date)
        
        if not actual_costs or not actual_usage:
            return None
        
        # Load our estimated data
        spending_file = self.data_dir / 'openai_daily_spending.json'
        if not spending_file.exists():
            log.warning("No estimated spending data found")
            return {
                'date': target_date.isoformat(),
                'actual': actual_costs,
                'estimated': None,
                'comparison': 'No estimates available'
            }
        
        with open(spending_file, 'r') as f:
            estimated_data = json.load(f)
        
        date_str = target_date.isoformat()
        estimated_day = estimated_data.get(date_str, {})
        
        if not estimated_day:
            return {
                'date': date_str,
                'actual': actual_costs,
                'estimated': None,
                'comparison': f'No estimates for {date_str}'
            }
        
        estimated_cost = estimated_day.get('total_cost_usd', 0)
        actual_cost = actual_costs['total_cost_usd']
        
        if estimated_cost > 0:
            accuracy_percent = (1 - abs(actual_cost - estimated_cost) / estimated_cost) * 100
            if actual_cost > estimated_cost:
                comparison = f"Underestimated by ${actual_cost - estimated_cost:.4f} ({accuracy_percent:.1f}% accurate)"
            else:
                comparison = f"Overestimated by ${estimated_cost - actual_cost:.4f} ({accuracy_percent:.1f}% accurate)"
        else:
            comparison = "No cost estimate available"
        
        return {
            'date': date_str,
            'actual': actual_costs,
            'estimated': {
                'total_cost_usd': estimated_cost,
                'total_input_tokens': estimated_day.get('total_input_tokens', 0),
                'total_output_tokens': estimated_day.get('total_output_tokens', 0)
            },
            'comparison': comparison
        }


def main():
    """CLI interface for the usage tracker"""
    import argparse
    
    parser = argparse.ArgumentParser(description='OpenAI Usage and Cost Tracker')
    parser.add_argument('--sync', action='store_true', help='Sync usage and cost data')
    parser.add_argument('--compare', action='store_true', help='Compare actual vs estimated costs')
    parser.add_argument('--summary', action='store_true', help='Show UTC-aligned daily summary')
    parser.add_argument('--date', type=str, help='Target date (YYYY-MM-DD), defaults to today')
    parser.add_argument('--days', type=int, default=7, help='Number of days to fetch (default: 7)')
    
    args = parser.parse_args()
    
    # Set up logging
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(levelname)s - %(message)s"
    )
    
    try:
        tracker = OpenAIUsageTracker()
        
        target_date = None
        if args.date:
            target_date = datetime.strptime(args.date, '%Y-%m-%d').date()
        
        if args.sync:
            result = tracker.sync_daily_data(target_date)
            print(f"\n📊 OpenAI Data Sync Results")
            print("=" * 40)
            
            if result['usage']:
                usage = result['usage']
                print(f"📅 Date: {usage['date']}")
                print(f"🔢 Tokens: {usage['total_input_tokens']:,} input + {usage['total_output_tokens']:,} output")
                print(f"⚡ Requests: {usage['total_requests']:,}")
            
            if result['costs']:
                costs = result['costs']
                print(f"💰 Total Cost: ${costs['total_cost_usd']:.4f}")
                
                if costs['breakdown']:
                    print("\n📋 Cost Breakdown:")
                    for item, cost in sorted(costs['breakdown'].items(), key=lambda x: x[1], reverse=True):
                        print(f"   {item}: ${cost:.4f}")
        
        elif args.compare:
            comparison = tracker.compare_with_estimates(target_date)
            if comparison:
                print(f"\n🔍 Actual vs Estimated Comparison")
                print("=" * 40)
                print(f"📅 Date: {comparison['date']}")
                print(f"💰 Actual Cost: ${comparison['actual']['total_cost_usd']:.4f}")
                
                if comparison['estimated']:
                    print(f"📊 Estimated Cost: ${comparison['estimated']['total_cost_usd']:.4f}")
                    print(f"🎯 Accuracy: {comparison['comparison']}")
                else:
                    print("📊 No estimates available for comparison")
            else:
                print("❌ No data available for comparison")
        
        elif args.summary:
            summary = tracker.get_utc_daily_summary(args.days)
            if summary:
                print(f"\n📊 UTC Daily Summary (Last {args.days} days)")
                print("=" * 60)
                print(f"🌍 Timezone: {summary[0]['timezone']}")
                print()
                
                for day in summary:
                    if day['total_cost'] > 0 or day['total_input_tokens'] > 0:
                        print(f"📅 {day['date']}")
                        print(f"   💰 Cost: ${day['total_cost']:.4f}")
                        print(f"   🔢 Tokens: {day['total_input_tokens']:,} input + {day['total_output_tokens']:,} output")
                        print(f"   ⚡ Requests: {day['total_requests']:,}")
                        print()
            else:
                print("❌ No summary data available")
        
        else:
            # Default: just fetch and display basic info
            tracker.fetch_usage_data(args.days)
            tracker.fetch_costs_data(args.days)
            print(f"✅ Fetched {args.days} days of usage and cost data")
    
    except Exception as e:
        log.error(f"Error: {e}")
        return 1
    
    return 0


if __name__ == '__main__':
    exit(main()) 