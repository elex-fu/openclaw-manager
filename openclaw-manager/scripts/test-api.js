// Test OpenClaw API in browser console
// Copy and paste this into the browser console (Cmd+Option+I)

async function testOpenClawAPI() {
  console.log('=== Testing OpenClaw API ===\n');

  try {
    // Test 1: Check installation
    console.log('1. check_openclaw_installation');
    const status = await invoke('check_openclaw_installation');
    console.log('   Result:', status);

    // Test 2: Check running status
    console.log('\n2. is_openclaw_running');
    const running = await invoke('is_openclaw_running');
    console.log('   Result:', running);
    if (running.data) {
      console.log('   - installed:', running.data.installed);
      console.log('   - running:', running.data.running);
      console.log('   - version:', running.data.version);
      console.log('   - install_path:', running.data.install_path);
    }

    // Test 3: Get process info
    console.log('\n3. get_openclaw_process_info');
    const processes = await invoke('get_openclaw_process_info');
    console.log('   Result:', processes);
    if (processes.data && processes.data.length > 0) {
      processes.data.forEach((p, i) => {
        console.log(`   Process ${i + 1}: PID=${p.pid}, name=${p.name}`);
      });
    } else {
      console.log('   No processes found');
    }

    // Test 4: Get config
    console.log('\n4. get_openclaw_config_if_installed');
    const config = await invoke('get_openclaw_config_if_installed');
    console.log('   Result:', config ? 'Config loaded' : 'No config');

    console.log('\n=== Test Complete ===');
  } catch (error) {
    console.error('Test failed:', error);
  }
}

testOpenClawAPI();
