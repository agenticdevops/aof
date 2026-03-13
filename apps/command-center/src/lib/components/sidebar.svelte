<script lang="ts">
	import { page } from '$app/state';
	import { cn } from '$lib/utils.js';
	import {
		LayoutDashboard,
		Bot,
		PlayCircle,
		DollarSign,
		Activity,
		ShieldCheck,
		Wrench,
		Settings,
		PanelLeftClose,
		PanelLeft
	} from 'lucide-svelte';
	import ThemeToggle from './theme-toggle.svelte';

	let collapsed = $state(false);
	let { mobileOpen = $bindable(false) }: { mobileOpen?: boolean } = $props();

	const links = [
		{ href: '/', label: 'Dashboard', icon: LayoutDashboard },
		{ href: '/agents', label: 'Agents', icon: Bot },
		{ href: '/runs', label: 'Runs', icon: PlayCircle },
		{ href: '/costs', label: 'Costs', icon: DollarSign },
		{ href: '/traces', label: 'Traces', icon: Activity },
		{ href: '/approvals', label: 'Approvals', icon: ShieldCheck },
		{ href: '/builder', label: 'Agent Builder', icon: Wrench }
	];

	function isActive(href: string): boolean {
		if (href === '/') return page.url.pathname === '/';
		return page.url.pathname.startsWith(href);
	}
</script>

<aside
	class={cn(
		'flex h-screen flex-col border-r bg-card transition-all duration-200',
		collapsed ? 'w-16' : 'w-60',
		'max-lg:fixed max-lg:z-40',
		!mobileOpen && 'max-lg:-translate-x-full'
	)}
>
	<div class="flex h-14 items-center border-b px-4">
		{#if !collapsed}
			<span class="text-lg font-semibold tracking-tight">OpenAgentiX</span>
		{/if}
		<button
			onclick={() => (collapsed = !collapsed)}
			class="ml-auto cursor-pointer rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors duration-200"
			aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
		>
			{#if collapsed}
				<PanelLeft class="h-4 w-4" />
			{:else}
				<PanelLeftClose class="h-4 w-4" />
			{/if}
		</button>
	</div>

	<nav class="flex-1 space-y-1 p-2">
		{#each links as link}
			<a
				href={link.href}
				onclick={() => (mobileOpen = false)}
				class={cn(
					'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors duration-200 cursor-pointer',
					isActive(link.href)
						? 'bg-accent text-accent-foreground'
						: 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
				)}
				title={collapsed ? link.label : undefined}
			>
				<link.icon class="h-4 w-4 shrink-0" />
				{#if !collapsed}
					<span>{link.label}</span>
				{/if}
			</a>
		{/each}
	</nav>

	<div class="border-t p-2 space-y-1">
		<a
			href="/settings"
			class={cn(
				'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors duration-200 cursor-pointer',
				isActive('/settings')
					? 'bg-accent text-accent-foreground'
					: 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
			)}
			title={collapsed ? 'Settings' : undefined}
		>
			<Settings class="h-4 w-4 shrink-0" />
			{#if !collapsed}
				<span>Settings</span>
			{/if}
		</a>
		<div class={cn('flex items-center', collapsed ? 'justify-center' : 'px-3')}>
			<ThemeToggle />
		</div>
	</div>
</aside>
