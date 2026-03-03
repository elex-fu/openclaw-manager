// Test OpenClaw API from browser console
// Run this in the browser console (Cmd+Option+I)

async function testOpenClawAPI() {
  console.log('=== Testing OpenClaw API ===\n');

  try {
    // Test 1: Check installation status
    console.log('1. Testing check_openclaw_installation...');
    const installStatus = await invoke('check_openclaw_installation');
    console.log('   Result:', installStatus);
    console.log('   ✓ Installation check completed\n');

    // Test 2: Check if running
    console.log('2. Testing is_openclaw_running...');
    const runningStatus = await invoke('is_openclaw_running');
    console.log('   Result:', runningStatus);
    if (runningStatus.data) {
      console.log('   Installed:', runningStatus.data.installed);
      console.log('   Running:', runningStatus.data.running);
      console.log('   Version:', runningStatus.data.version);
      console.log('   Install Path:', runningStatus.data.install_path);
    }
    console.log('   ✓ Running status check completed\n');

    // Test 3: Get process info
    console.log('3. Testing get_openclaw_process_info...');
    const processInfo = await invoke('get_openclaw_process_info');
    console.log('   Result:', processInfo);
    if (processInfo.data && processInfo.data.length > 0) {
      processInfo.data.forEach((proc, i) => {
        console.log(`   Process ${i + 1}:`, proc);
      });
    } else {
      console.log('   No processes found');
    }
    console.log('   ✓ Process info check completed\n');

    // Test 4: Get config (if installed)
    console.log('4. Testing get_openclaw_config_if_installed...');
    const config = await invoke('get_openclaw_config_if_installed');
    console.log('   Result:', config);
    if (config.data) {
      console.log('   ✓ Config loaded');
      console.log('   Models:', config.data.models?.length || 0);
      console.log('   Agents:', config.data.agents?.length || 0);
    } else {
      console.log('   ✗ No config (OpenClaw not installed)');
    }
    console.log('   ✓ Config check completed\n');

    // Test 5: Get models
    console.log('5. Testing get_openclaw_models...');
    const models = await invoke('get_openclaw_models');
    console.log('   Result:', models);
    console.log('   Model count:', models.data?.length || 0);
    console.log('   ✓ Models check completed\n');

    // Test 6: Get agents
    console.log('6. Testing get_openclaw_agents...');
    const agents = await invoke('get_openclaw_agents');
    console.log('   Result:', agents);
    console.log('   Agent count:', agents.data?.length || 0);
    console.log('   ✓ Agents check completed\n');

    console.log('=== All API Tests Completed ===');

  } catch (error) {
    console.error('✗ API Test failed:', error);
  }
}

// Run the test
testOpenClawAPI();
