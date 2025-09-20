#!/usr/bin/env python3
"""
SQLite Database Viewer for Oracle BRM Dashboard
Simple script to view and manage the users database
"""

import sqlite3
import sys
from datetime import datetime

DATABASE = 'users.db'

def get_db_connection():
    """Get database connection"""
    conn = sqlite3.connect(DATABASE)
    conn.row_factory = sqlite3.Row
    return conn

def show_all_users():
    """Display all users in a formatted table"""
    conn = get_db_connection()
    users = conn.execute('SELECT * FROM users ORDER BY created_at DESC').fetchall()
    conn.close()
    
    if not users:
        print("No users found in database.")
        return
    
    print("\n" + "="*100)
    print("ORACLE BRM DASHBOARD - USER DATABASE")
    print("="*100)
    print(f"{'Username':<15} {'Email':<30} {'Role':<8} {'Status':<8} {'Last Login':<20} {'Created':<20}")
    print("-"*100)
    
    for user in users:
        last_login = user['last_login'] if user['last_login'] else 'Never'
        created = user['created_at'][:19] if user['created_at'] else 'Unknown'
        print(f"{user['username']:<15} {user['email']:<30} {user['role']:<8} {user['status']:<8} {last_login:<20} {created:<20}")
    
    print("="*100)
    print(f"Total users: {len(users)}")

def show_user_details(username):
    """Show detailed information for a specific user"""
    conn = get_db_connection()
    user = conn.execute('SELECT * FROM users WHERE username = ?', (username,)).fetchone()
    conn.close()
    
    if not user:
        print(f"User '{username}' not found.")
        return
    
    print(f"\nUser Details for: {username}")
    print("-" * 50)
    print(f"Username: {user['username']}")
    print(f"Email: {user['email']}")
    print(f"Role: {user['role']}")
    print(f"Status: {user['status']}")
    print(f"Last Login: {user['last_login'] if user['last_login'] else 'Never'}")
    print(f"Created: {user['created_at']}")
    print(f"Password Hash: {user['password'][:50]}...")

def show_database_stats():
    """Show database statistics"""
    conn = get_db_connection()
    
    # Total users
    total_users = conn.execute('SELECT COUNT(*) FROM users').fetchone()[0]
    
    # Users by role
    admin_count = conn.execute('SELECT COUNT(*) FROM users WHERE role = ?', ('admin',)).fetchone()[0]
    user_count = conn.execute('SELECT COUNT(*) FROM users WHERE role = ?', ('user',)).fetchone()[0]
    
    # Active/Inactive users
    active_count = conn.execute('SELECT COUNT(*) FROM users WHERE status = ?', ('active',)).fetchone()[0]
    inactive_count = conn.execute('SELECT COUNT(*) FROM users WHERE status = ?', ('inactive',)).fetchone()[0]
    
    # Recent logins
    recent_logins = conn.execute('SELECT COUNT(*) FROM users WHERE last_login IS NOT NULL').fetchone()[0]
    
    conn.close()
    
    print("\nDatabase Statistics")
    print("-" * 30)
    print(f"Total Users: {total_users}")
    print(f"  - Admins: {admin_count}")
    print(f"  - Regular Users: {user_count}")
    print(f"Active Users: {active_count}")
    print(f"Inactive Users: {inactive_count}")
    print(f"Users with Login History: {recent_logins}")


def interactive_mode():
    """Interactive database viewer"""
    while True:
        print("\n" + "="*50)
        print("ORACLE BRM DASHBOARD - DATABASE VIEWER")
        print("="*50)
        print("📊 VIEW OPTIONS:")
        print("  1. Show all users")
        print("  2. Show user details")
        print("  3. Show database statistics")
        print("")
        print("  4. Exit")
        print("-"*50)
        
        choice = input("Enter your choice (1-4): ").strip()
        
        if choice == '1':
            show_all_users()
        elif choice == '2':
            username = input("Enter username: ").strip()
            show_user_details(username)
        elif choice == '3':
            show_database_stats()
        elif choice == '4':
            print("Goodbye!")
            break
        else:
            print("❌ Invalid choice. Please try again.")

def main():
    """Main function"""
    if len(sys.argv) > 1:
        command = sys.argv[1].lower()
        
        if command == 'list' or command == 'all':
            show_all_users()
        elif command == 'stats':
            show_database_stats()
        elif command == 'user' and len(sys.argv) > 2:
            show_user_details(sys.argv[2])
        else:
            print("ORACLE BRM DASHBOARD - DATABASE VIEWER")
            print("="*50)
            print("Usage:")
            print("  python db_viewer.py list     - Show all users")
            print("  python db_viewer.py stats    - Show database statistics")
            print("  python db_viewer.py user <username> - Show user details")
            print("  python db_viewer.py          - Interactive mode")
    else:
        interactive_mode()

if __name__ == "__main__":
    main()
